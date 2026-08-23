--
-- PostgreSQL database dump
--

\restrict RARgkVVMvLptmcpnTFpaEszHLkUhzkjkTLBtIkfWEytrxGFtuP7hh3OVaSf8NY8

-- Dumped from database version 16.14 (Debian 16.14-1.pgdg12+1)
-- Dumped by pg_dump version 18.1

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public._sqlx_migrations VALUES (1, 'initial core storage', '2026-08-23 08:40:21.714393+00', true, '\xffd41e96b85720283063204d17f5efc4c770edfba3afe930e8c538170d8b10bb2f550f66de92b798fcad5dbc18a010a7', 13781833);
INSERT INTO public._sqlx_migrations VALUES (2, 'knowledge rag graph schema', '2026-08-23 08:40:21.729166+00', true, '\x834b992ef891a88d5e98ac33984d0e433ac95514d7bd899a2af896c3c3537168f79904874965e3673887a839dfb55738', 10709917);
INSERT INTO public._sqlx_migrations VALUES (3, 'agent skills registry schema', '2026-08-23 08:40:21.740318+00', true, '\xcbd908f01a7f41a5e3458ab6cb6f21cdf2c9dd9a51e40b910799a5d638f893bdb2d1593056a272c622485641f1009884', 8886875);
INSERT INTO public._sqlx_migrations VALUES (4, 'mcp servers schema', '2026-08-23 08:40:21.749877+00', true, '\x39d7ba4ed8d4788868fd07c58cfa8ae6dccab767f0faeac025c1d213f425f0a5eb25711e3a33d6fb20e3a17206858e50', 6893167);
INSERT INTO public._sqlx_migrations VALUES (5, 'agent test judge engine', '2026-08-23 08:40:21.75723+00', true, '\x9072862792bd8097d2c38801b50db3d52e901b4f0e9701a6d485b67de029990cbee5660ef3c2a41fdd766b51a800a798', 8870000);
INSERT INTO public._sqlx_migrations VALUES (6, 'trait definitions schema', '2026-08-23 08:40:21.766586+00', true, '\x7d848bb50a2d1688bae7fa23652535cd2400302656dc3d11993e61038e7ab885ed0b9edb5cb6e445655e30c229b7ef79', 6804667);
INSERT INTO public._sqlx_migrations VALUES (7, 'add guardrail config to agents', '2026-08-23 08:40:21.773824+00', true, '\x94e6c1f6605b94c703be0e70bd75902a0e8c72dbd666df9a4c44a6ab55edaafab7bb468f269346f6766dcbb808f92038', 1397500);
INSERT INTO public._sqlx_migrations VALUES (8, 'add archived at to agents', '2026-08-23 08:40:21.775671+00', true, '\xed929bc142086cfeeb48c9a849a67b3fdfd5a15757d2670508c2526494976b98ccc2c5740a25806219468898f36cc503', 2777167);
INSERT INTO public._sqlx_migrations VALUES (9, 'add definition to skills', '2026-08-23 08:40:21.778871+00', true, '\xd4f9a33482197409cf8680bdab3c5adc4933c2c542257f6d9faa69ec02b14884d750a2ebedf77dee43433640c6ec309d', 1240917);
INSERT INTO public._sqlx_migrations VALUES (10, 'add references to skills', '2026-08-23 08:40:21.780476+00', true, '\x7d4863cbb26892b025880c3fadc361acc5a1dac609e986f8304c3f469814c557bf1f978c5d97f7897d8ab6b5b4454361', 1456333);
INSERT INTO public._sqlx_migrations VALUES (11, 'update agent references', '2026-08-23 08:40:21.782427+00', true, '\xfddb2d2fd08b88039e5b05e61033b0bfc5f5dcc84ace4d39b033a0f040298c984a2edeaea4353dccd5ae57db2f7d7e45', 3171000);
INSERT INTO public._sqlx_migrations VALUES (12, 'migrate version to semver', '2026-08-23 08:40:21.786863+00', true, '\xe7dcfc2314f08a207e313a3a917e855e0c31826bbf51695bf8666a1d7945a66cd7fb2fa4072607f30369ec818e6f33d9', 27611000);
INSERT INTO public._sqlx_migrations VALUES (13, 'add implements traits to skills', '2026-08-23 08:40:21.814735+00', true, '\x867cd3b901458d18ec5727580f74ece1fc8eaf77da71d6865b4f5fcc4b869e931150e8a4e5570b6914f35a701b01ce78', 1452083);
INSERT INTO public._sqlx_migrations VALUES (14, 'add owner id to mcp servers and traits', '2026-08-23 08:40:21.816581+00', true, '\x32313ec5f9be9f2f6f1a986723d38a5b13a2237f9c1df260226f8831eb1c19a4f6b2de0cfb3df67238df5e05167a63ed', 1216375);
INSERT INTO public._sqlx_migrations VALUES (15, 'rename mcp servers to tools', '2026-08-23 10:08:02.439232+00', true, '\x20c735312af82c6385c021c955a208183d0c19659cf80c453dd375a5e51470b6ce5e81c87772bf3b445629ad78777c89', 2511458);


--
-- Data for Name: agents; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public.agents VALUES ('ea1ee801-5fad-4257-93fd-e16aa37492dc', 'Journey7_Judge_Agent', '', '{}', '{}', '1.1.0', '11111111-1111-1111-1111-111111111111', '{}', '{}', '{}', '[]', '[]', '{"role": "tester updated"}', 'null', '[]', '[]', '[]', '2026-08-23 10:31:32.628387+00', '2026-08-23 10:31:32.916342+00', 0.8, NULL, '2026-08-23 10:31:32.916342+00', '{}', '{}', '{}');
INSERT INTO public.agents VALUES ('46a7f559-7b5b-424f-a050-7a86da29b000', 'Journey12_Soft_Delete_Agent_1', '', '{}', '{}', '1.0.0', '22222222-2222-2222-2222-222222222222', '{}', '{}', '{}', '[]', '[]', '{"role": "tester-soft-delete"}', 'null', '[]', '[]', '[]', '2026-08-23 10:31:36.337894+00', '2026-08-23 10:31:36.416121+00', 0, NULL, '2026-08-23 10:31:36.416121+00', '{}', '{}', '{}');
INSERT INTO public.agents VALUES ('b2417ab0-94af-4252-9c79-02b9acca0db2', 'BasicSecurityTest', 'Create a simple assessment of the codebase and penetration test of the applications having prior knowledge of the codebase.', '{ben,security,code}', '{SecurityAudit}', '1.0.0', '00000000-0000-0000-0000-000000000000', '{}', '{}', '{}', '[]', '[]', '"# Vicious Test\n\nHaving access to the code make a malitious attack using code and run it against the deployed systems."', '"claude-3-5-sonnet-v2"', '[]', '[]', '[]', '2026-08-23 08:45:55.283769+00', '2026-08-23 20:48:33.830097+00', 0.8, '{"input_guardrails": {"active_guardrails": []}, "output_guardrails": {"active_guardrails": []}}', NULL, '{649f8644-7383-4261-a03f-71dd5f8e18e6}', '{3e68e6f2-595d-42a8-93e8-32b769710d66}', '{}');


--
-- Data for Name: agent_embeddings; Type: TABLE DATA; Schema: public; Owner: postgres
--



--
-- Data for Name: agent_revisions; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public.agent_revisions VALUES ('599eb8e3-8cc9-4815-a40e-d849b99763ef', 'b2417ab0-94af-4252-9c79-02b9acca0db2', '1.0.0', '{"id": "b2417ab0-94af-4252-9c79-02b9acca0db2", "name": "BenSecurityTest", "version": "1.0.0", "agent_definition": "# Vicious Test\n\nHaving access to the code make a malitious attack using code and run it against the deployed systems."}', '2026-08-23 08:45:55.292612+00');
INSERT INTO public.agent_revisions VALUES ('6d73c970-1aa5-4494-857f-7f5a9fc546e1', 'ea1ee801-5fad-4257-93fd-e16aa37492dc', '1.0.0', '{"id": "ea1ee801-5fad-4257-93fd-e16aa37492dc", "name": "Journey7_Judge_Agent", "version": "1.0.0", "agent_definition": {"role": "tester"}}', '2026-08-23 10:31:32.770535+00');
INSERT INTO public.agent_revisions VALUES ('3e6c055b-8da9-40b5-9be4-339fcdb58ab9', 'ea1ee801-5fad-4257-93fd-e16aa37492dc', '1.1.0', '{"id": "ea1ee801-5fad-4257-93fd-e16aa37492dc", "name": "Journey7_Judge_Agent", "version": "1.1.0", "agent_definition": {"role": "tester updated"}}', '2026-08-23 10:31:32.887959+00');
INSERT INTO public.agent_revisions VALUES ('e3e9e88b-cbc8-41ed-a00f-9cda5dbea4c2', '46a7f559-7b5b-424f-a050-7a86da29b000', '1.0.0', '{"id": "46a7f559-7b5b-424f-a050-7a86da29b000", "name": "Journey12_Soft_Delete_Agent_1", "version": "1.0.0", "agent_definition": {"role": "tester-soft-delete"}}', '2026-08-23 10:31:36.341848+00');


--
-- Data for Name: agent_test_suites; Type: TABLE DATA; Schema: public; Owner: postgres
--



--
-- Data for Name: agent_test_runs; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public.agent_test_runs VALUES ('d052b5fe-82c9-4c52-9e3d-dbe3a68ff915', 'ea1ee801-5fad-4257-93fd-e16aa37492dc', '1.0.0', NULL, 'regression_blocked', '{}', '{"threshold": 0.95, "average_score": 0.9, "test_cases_evaluated": 1}', '2026-08-23 10:31:32.846544+00');
INSERT INTO public.agent_test_runs VALUES ('01809581-2887-40de-83bf-ceecd04cd115', 'ea1ee801-5fad-4257-93fd-e16aa37492dc', '1.0.0', NULL, 'passed', '{}', '{"threshold": 0.8, "average_score": 0.9, "test_cases_evaluated": 1}', '2026-08-23 10:31:32.892272+00');


--
-- Data for Name: executions; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public.executions VALUES ('a4b4a5d7-87bd-4d29-93ef-5a297f082d8a', 'ea1ee801-5fad-4257-93fd-e16aa37492dc', '1.1.0', 1, 'completed', '{}', '{"prompt": "run diagnostic checks"}', '{"output": "Executed agent response for prompt: ''run diagnostic checks''"}', NULL, NULL, '2026-08-23 10:31:32.904193+00', '2026-08-23 10:31:32.904193+00', '2026-08-23 10:31:32.904193+00');
INSERT INTO public.executions VALUES ('2711ee42-8b9e-40fc-aa19-dd3f1ce79fbc', '46a7f559-7b5b-424f-a050-7a86da29b000', '1.0.0', 1, 'completed', '{}', '{"prompt": "trigger test execution"}', '{"output": "Executed agent response for prompt: ''trigger test execution''"}', NULL, NULL, '2026-08-23 10:31:36.405336+00', '2026-08-23 10:31:36.405336+00', '2026-08-23 10:31:36.405336+00');


--
-- Data for Name: knowledge_nodes; Type: TABLE DATA; Schema: public; Owner: postgres
--



--
-- Data for Name: knowledge_embeddings; Type: TABLE DATA; Schema: public; Owner: postgres
--



--
-- Data for Name: knowledge_tuples; Type: TABLE DATA; Schema: public; Owner: postgres
--



--
-- Data for Name: skills; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public.skills VALUES ('89817d05-7320-4874-ae82-6a06e0f78c10', 'Quick', 'i do a lot of stuff and try to do many things.', '{security}', '1.3.0', '00000000-0000-0000-0000-000000000000', '{}', '{}', '{}', '{}', '{}', '2026-08-23 09:57:36.515027+00', '2026-08-23 20:47:53.85534+00', '# Busy

Get busy and jsut do stuff.', '{}', '{3e68e6f2-595d-42a8-93e8-32b769710d66}', '{}');
INSERT INTO public.skills VALUES ('649f8644-7383-4261-a03f-71dd5f8e18e6', 'Useless', 'A simple skills', '{}', '1.3.0', '00000000-0000-0000-0000-000000000000', '{}', '{}', '{}', '{}', '{}', '2026-08-23 08:44:24.821124+00', '2026-08-23 20:48:07.418816+00', '# Do very little', '{}', '{}', '{PenTest}');


--
-- Data for Name: tools; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public.tools VALUES ('3e68e6f2-595d-42a8-93e8-32b769710d66', 'test-mcp-server-fixed', 'sse', '{"url": "http://localhost:3000/sse", "tags": [], "description": ""}', '{"tools": [{"name": "search_agents", "description": "RAG search for matching agents"}, {"name": "execute_agent", "description": "Run agent with payload"}], "prompts": [], "resources": []}', '2026-08-23 08:52:00.878052+00', '00000000-0000-0000-0000-000000000001');


--
-- Data for Name: trait_contracts; Type: TABLE DATA; Schema: public; Owner: postgres
--

INSERT INTO public.trait_contracts VALUES ('41419e76-0bce-4202-89fd-7354abba06b4', 'SecurityAudit', 'Run a security audit checking for vulnerabilities and bad configurations.', '1.0.0', '{}', '{}', '{}', '{security,audit}', '{"input_guardrails": {"active_guardrails": []}, "output_guardrails": {"active_guardrails": []}}', '2026-08-23 08:42:01.008582+00', '2026-08-23 08:42:01.008582+00', '00000000-0000-0000-0000-000000000000');
INSERT INTO public.trait_contracts VALUES ('18808059-21a3-48e2-8ae7-71c14586da5a', 'PenTest', 'Run a Security Penetration test to assess if known vulnerabilities are exploitable in the deployment.', '1.0.0', '{}', '{}', '{}', '{security,test}', '{"input_guardrails": {"active_guardrails": []}, "output_guardrails": {"active_guardrails": []}}', '2026-08-23 08:42:52.688038+00', '2026-08-23 08:42:52.688038+00', '00000000-0000-0000-0000-000000000000');


--
-- PostgreSQL database dump complete
--

\unrestrict RARgkVVMvLptmcpnTFpaEszHLkUhzkjkTLBtIkfWEytrxGFtuP7hh3OVaSf8NY8

