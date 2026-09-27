#!/usr/bin/env python3
"""
cleanup_stale_data.py - Clean up duplicate, near-duplicate, and stale test data via Agent-as-Data REST APIs.

Usage:
    python3 cleanup_stale_data.py [--dry-run] [--api-url http://localhost:8080/api/v1]
"""

import json
import re
import urllib.error
import urllib.request
from collections import defaultdict

import click

DEFAULT_BASE_URL = "http://localhost:8080/api/v1"


class ApiClient:
    def __init__(self, base_url: str = DEFAULT_BASE_URL, dry_run: bool = False):
        self.base_url = base_url.rstrip("/")
        self.dry_run = dry_run

    def request(self, method: str, path: str, data=None):
        url = f"{self.base_url}{path}"
        headers = {}
        body = None
        if data is not None:
            body = json.dumps(data).encode("utf-8")
            headers["Content-Type"] = "application/json"

        req = urllib.request.Request(url, data=body, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req) as resp:
                status = resp.status
                resp_text = resp.read().decode("utf-8")
                if resp_text:
                    try:
                        return status, json.loads(resp_text)
                    except json.JSONDecodeError:
                        return status, resp_text
                return status, None
        except urllib.error.HTTPError as e:
            err_body = e.read().decode("utf-8")
            return e.code, err_body
        except Exception as e:
            return 500, str(e)


def clean_knowledge(client: ApiClient):
    click.echo("\n🔍 Checking Knowledge Assets...")
    status, nodes = client.request("GET", "/knowledge")
    if status != 200 or not isinstance(nodes, list):
        click.secho(f"  ❌ Failed to fetch knowledge nodes: status {status} - {nodes}", fg="red")
        return

    click.echo(f"  Found {len(nodes)} knowledge node(s).")
    groups = defaultdict(list)
    for node in nodes:
        key = node.get("title") or node.get("topic") or "Untitled"
        groups[key].append(node)

    deleted_count = 0
    retained_count = 0

    for key, items in groups.items():
        sorted_items = sorted(items, key=lambda x: x.get("created_at") or "")
        duplicates = sorted_items[1:]

        retained_count += 1
        if duplicates:
            click.secho(f"  Found {len(duplicates)} duplicate(s) for knowledge '{key}'", fg="yellow")
            for dup in duplicates:
                dup_id = dup["id"]
                if client.dry_run:
                    click.echo(f"    [DRY-RUN] Would delete duplicate knowledge node {dup_id} ('{key}')")
                    deleted_count += 1
                else:
                    d_status, _ = client.request("DELETE", f"/knowledge/{dup_id}")
                    if d_status in (200, 204):
                        click.secho(f"    ✅ Deleted duplicate knowledge node {dup_id} ('{key}')", fg="green")
                        deleted_count += 1
                    else:
                        click.secho(f"    ❌ Failed to delete knowledge node {dup_id}: status {d_status}", fg="red")

    click.echo(f"  Knowledge summary: {deleted_count} removed, {retained_count} retained.")


def clean_agents(client: ApiClient):
    click.echo("\n🔍 Checking Agent Assets...")
    status, agents = client.request("POST", "/agents/search", {"query": "", "limit": 2000})
    if status != 200 or not isinstance(agents, list):
        click.secho(f"  ❌ Failed to search agents: status {status} - {agents}", fg="red")
        return

    click.echo(f"  Found {len(agents)} active agent(s).")

    groups = defaultdict(list)
    journey_test_agents = []

    for agent in agents:
        agent_id = agent.get("id") or agent.get("agent_id")
        name = agent.get("name", "")

        if (
            re.match(r"^Journey\d+_Agent_[A-Za-z0-9]+$", name)
            or re.match(r"^Journey\d+_Soft_Delete_Agent", name)
            or re.match(r"^Journey\d+_Judge_Agent", name)
            or re.match(r"^Journey\d+_Rust_Security_Auditor", name)
        ):
            journey_test_agents.append((agent_id, name))
        else:
            groups[name].append((agent_id, agent))

    hard_del_count = 0
    soft_del_count = 0
    retained_count = 0

    # Clean exact duplicate exemplar agents
    for name, items in groups.items():
        if len(items) <= 1:
            retained_count += 1
            continue

        click.secho(f"  Found {len(items)} duplicates for agent '{name}'. Keeping primary instance...", fg="yellow")
        duplicates = items[1:]
        retained_count += 1

        for dup_id, _ in duplicates:
            if client.dry_run:
                click.echo(f"    [DRY-RUN] Would delete agent duplicate {dup_id} ('{name}')")
                hard_del_count += 1
            else:
                del_status, resp = client.request("DELETE", f"/agents/{dup_id}?hard=true")
                if del_status == 200:
                    click.secho(f"    ✅ Hard-deleted duplicate agent {dup_id} ('{name}')", fg="green")
                    hard_del_count += 1
                elif del_status == 409:
                    s_status, _ = client.request("DELETE", f"/agents/{dup_id}")
                    if s_status == 200:
                        click.secho(f"    ⚠️ Agent has execution history; soft-deleted (archived) {dup_id} ('{name}')", fg="yellow")
                        soft_del_count += 1
                    else:
                        click.secho(f"    ❌ Failed to soft delete agent {dup_id}", fg="red")
                else:
                    click.secho(f"    ❌ Failed to delete agent {dup_id}: status {del_status} - {resp}", fg="red")

    # Clean near-duplicate test journey agents
    if journey_test_agents:
        click.secho(f"  Found {len(journey_test_agents)} stale test journey agent(s). Cleaning...", fg="yellow")
        for agent_id, name in journey_test_agents:
            if client.dry_run:
                click.echo(f"    [DRY-RUN] Would delete test agent {agent_id} ('{name}')")
                hard_del_count += 1
            else:
                del_status, resp = client.request("DELETE", f"/agents/{agent_id}?hard=true")
                if del_status == 200:
                    click.secho(f"    ✅ Hard-deleted test agent {agent_id} ('{name}')", fg="green")
                    hard_del_count += 1
                elif del_status == 409:
                    s_status, _ = client.request("DELETE", f"/agents/{agent_id}")
                    if s_status == 200:
                        click.secho(f"    ⚠️ Test agent has executions; soft-deleted (archived) {agent_id} ('{name}')", fg="yellow")
                        soft_del_count += 1
                    else:
                        click.secho(f"    ❌ Failed to soft delete test agent {agent_id}", fg="red")
                else:
                    click.secho(f"    ❌ Failed to delete test agent {agent_id}: status {del_status} - {resp}", fg="red")

    click.echo(f"  Agents summary: {hard_del_count} hard-deleted, {soft_del_count} soft-deleted, {retained_count} active retained.")


def clean_benches(client: ApiClient):
    click.echo("\n🔍 Checking Workbench Benches...")
    status, benches = client.request("POST", "/benches", {
        "owner_id": "00000000-0000-0000-0000-000000000000",
        "pagination": {"page": 0, "size": 200}
    })
    if status != 200 or not isinstance(benches, list):
        click.secho(f"  ❌ Failed to fetch benches: status {status} - {benches}", fg="red")
        return

    click.echo(f"  Found {len(benches)} bench(es).")
    deleted_count = 0
    retained_count = 0

    for bench in benches:
        bid = bench["id"]
        name = bench.get("name", "")
        desc = bench.get("description", "") or ""

        is_test_bench = re.match(r"^MemoryBench_[A-Za-z0-9]+$", name) or "Integration test bench" in desc

        if is_test_bench:
            if client.dry_run:
                click.echo(f"    [DRY-RUN] Would delete test bench {bid} ('{name}')")
                deleted_count += 1
            else:
                d_status, _ = client.request("DELETE", f"/benches/{bid}")
                if d_status in (200, 204):
                    click.secho(f"    ✅ Deleted test bench {bid} ('{name}')", fg="green")
                    deleted_count += 1
                else:
                    click.secho(f"    ❌ Failed to delete test bench {bid}: status {d_status}", fg="red")
        else:
            retained_count += 1

    click.echo(f"  Benches summary: {deleted_count} removed, {retained_count} retained.")


def clean_threads(client: ApiClient):
    click.echo("\n🔍 Checking Threads...")
    status, threads = client.request("POST", "/threads", {
        "owner_id": "00000000-0000-0000-0000-000000000000",
        "pagination": {"page": 0, "size": 200}
    })
    if status != 200 or not isinstance(threads, list):
        click.secho(f"  ❌ Failed to fetch threads: status {status} - {threads}", fg="red")
        return

    click.echo(f"  Found {len(threads)} thread(s).")
    deleted_count = 0
    retained_count = 0

    for thread in threads:
        tid = thread["id"]
        title = thread.get("title", "")

        m_status, messages = client.request("GET", f"/threads/{tid}/messages")
        msg_count = len(messages) if (m_status == 200 and isinstance(messages, list)) else 0

        is_empty_test_thread = (msg_count == 0 and title in ("Thread 2", "General", "New Thread"))

        if is_empty_test_thread:
            if client.dry_run:
                click.echo(f"    [DRY-RUN] Would delete empty test thread {tid} ('{title}')")
                deleted_count += 1
            else:
                d_status, _ = client.request("DELETE", f"/threads/{tid}")
                if d_status in (200, 204):
                    click.secho(f"    ✅ Deleted empty test thread {tid} ('{title}')", fg="green")
                    deleted_count += 1
                else:
                    click.secho(f"    ❌ Failed to delete thread {tid}: status {d_status}", fg="red")
        else:
            retained_count += 1

    click.echo(f"  Threads summary: {deleted_count} removed, {retained_count} retained.")


def clean_skills(client: ApiClient):
    click.echo("\n🔍 Checking Skills...")
    status, skills = client.request("GET", "/skills")
    if status != 200 or not isinstance(skills, list):
        click.secho(f"  ❌ Failed to fetch skills: status {status} - {skills}", fg="red")
        return

    click.echo(f"  Found {len(skills)} skill(s).")
    groups = defaultdict(list)
    for skill in skills:
        name = skill.get("name", "")
        groups[name].append(skill)

    deleted_count = 0
    retained_count = 0

    for name, items in groups.items():
        if len(items) <= 1:
            retained_count += 1
            continue

        click.secho(f"  Found {len(items)} duplicates for skill '{name}'", fg="yellow")
        duplicates = items[1:]
        retained_count += 1
        for dup in duplicates:
            sid = dup["id"]
            if client.dry_run:
                click.echo(f"    [DRY-RUN] Would delete duplicate skill {sid} ('{name}')")
                deleted_count += 1
            else:
                d_status, _ = client.request("DELETE", f"/skills/{sid}")
                if d_status in (200, 204):
                    click.secho(f"    ✅ Deleted duplicate skill {sid} ('{name}')", fg="green")
                    deleted_count += 1
                else:
                    click.secho(f"    ❌ Failed to delete skill {sid}: status {d_status}", fg="red")

    click.echo(f"  Skills summary: {deleted_count} removed, {retained_count} retained.")


def clean_tools(client: ApiClient):
    click.echo("\n🔍 Checking Tools...")
    status, tools = client.request("GET", "/agents/tools")
    if status != 200 or not isinstance(tools, list):
        click.secho(f"  ❌ Failed to fetch tools: status {status} - {tools}", fg="red")
        return

    click.echo(f"  Found {len(tools)} tool server(s).")
    groups = defaultdict(list)
    for tool in tools:
        name = tool.get("server_name", "")
        groups[name].append(tool)

    deleted_count = 0
    retained_count = 0

    for name, items in groups.items():
        if len(items) <= 1:
            retained_count += 1
            continue

        click.secho(f"  Found {len(items)} duplicates for tool '{name}'", fg="yellow")
        duplicates = items[1:]
        retained_count += 1
        for dup in duplicates:
            tid = dup["id"]
            if client.dry_run:
                click.echo(f"    [DRY-RUN] Would delete duplicate tool {tid} ('{name}')")
                deleted_count += 1
            else:
                d_status, _ = client.request("DELETE", f"/agents/tools/{tid}")
                if d_status in (200, 204):
                    click.secho(f"    ✅ Deleted duplicate tool {tid} ('{name}')", fg="green")
                    deleted_count += 1
                else:
                    click.secho(f"    ❌ Failed to delete tool {tid}: status {d_status}", fg="red")

    click.echo(f"  Tools summary: {deleted_count} removed, {retained_count} retained.")


@click.command()
@click.option(
    "--api-url",
    default=DEFAULT_BASE_URL,
    show_default=True,
    help="Base API URL for the Agent-as-Data REST services.",
)
@click.option(
    "--dry-run",
    is_flag=True,
    help="Preview deletions without calling DELETE APIs.",
)
def main(api_url: str, dry_run: bool):
    """Clean up stale, duplicate, and near-duplicate test data via REST APIs."""
    click.secho("==================================================", fg="cyan", bold=True)
    click.secho("  Agent-as-Data Stale Test Data Cleanup", fg="cyan", bold=True)
    click.echo(f"  API URL: {api_url}")
    click.echo(f"  Dry Run: {'YES' if dry_run else 'NO'}")
    click.secho("==================================================", fg="cyan", bold=True)

    client = ApiClient(base_url=api_url, dry_run=dry_run)

    clean_knowledge(client)
    clean_agents(client)
    clean_benches(client)
    clean_threads(client)
    clean_skills(client)
    clean_tools(client)

    click.secho("\n✅ Stale test data cleanup scan finished.", fg="green", bold=True)


if __name__ == "__main__":
    main()
