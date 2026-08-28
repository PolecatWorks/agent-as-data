1. **Update Configuration Structure**:
   - In `aad-be-container/src/config.rs`, add `pub api_prefix: String,` to `WebServiceConfig`.
   - Update `aad-be-container/config/default.yaml` to include `api_prefix: "/api"` under `webservice`.
   - Update `AppConfig` tests in `config.rs` to include `api_prefix: "/api".to_string()`.

2. **Refactor Route Setup in `main.rs`**:
   - Modify `aad-be-container/src/main.rs` to group API routes under an `axum::Router::new().nest(&config.webservice.api_prefix, api_routes)`.
   - Ensure the `/health` route is added to the root app router independently, as it should not be affected by the API prefix.
   - The prefix in the configuration (e.g., `/api`) will combine with `/v1/...` which we'll define in the nest router to create the final route paths. (Wait, let's look at how the prefix should be formatted. The instructions say "uses a customisable prefix ... this prefix is used to prefix all the served APIs". If the frontend proxy expects `/api`, it means `api_prefix: "/api"`, and routes will be `/v1/...` relative to it, resulting in `/api/v1/...`).

3. **Update PRDs/Specs (Already Done)**:
   - PRDs were already updated to use `{{api_prefix}}`.

4. **Verify Frontend**:
   - The frontend proxy is already set up to proxy `/api` to `localhost:8080`, and the `ApiService` in `aad-fe-container/src/app/services/api.service.ts` uses `/api/v1`. If we configure the backend to use `/api` as the prefix, the frontend will continue to work without changes.

5. **Verify Tests**:
   - The integration tests use `/api/v1/...`. They will continue to work if the backend uses `/api` as the prefix.
   - Run integration tests to ensure nothing breaks.

6. **Pre-commit**:
   - Run `pre_commit_instructions`.

7. **Submit Changes**.
