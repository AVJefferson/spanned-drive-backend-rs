# Agent Readme For SDRIVE Backend

This file is used to give AI agents context about the project and how to use it.
AI Agents may edit this file to add more context about the project.

## Folder Structure

- .cargo/ -> DO NOT EDIT.
  - config.toml

- allowed_clients/ -> DO NOT EDIT. 400/600 permissions required to be loaded into the backend.
  - test.key -> nonprivileged client key file.
  - admin.key -> privilaged client key file.

- docker/ -> DO NOT EDIT.
  - docker-compose.yml -> The docker compose file for the project.
  - dockerfile -> The dockerfile for the project.

- target/ -> DO NOT EDIT.

- .dockerignore -> Docker Ignored files
- .gitignore -> Git Ignored files

- Cargo.toml -> Cargo configuration.
- Cargo.lock -> Cargo lock file.

- \*.env files -> Environment files for different environments (local, dev, stg, prd). DO NOT EDIT.

- README.md -> Readme for the project.
- agent.md -> Agent readme for the project.

- src/ -> Source code for the project. Most mod.rs files simply export the module; unless otherwise specified.
  - compile_time_checks/ -> Compile time checks for the project.
    - feature_check.rs -> ensures that only one environment is enabled at a time.

  - external_systems/ -> external systems called.
    - mod.rs -> exports external as client struct. External systems initialized from environment variables.
    - google/ -> Google for the project.
      - mod.rs -> exports google client.
      - config.rs -> Google client configurations.

  - logger/ -> Logger for the project. DO NOT EDIT.

  - middlewares/
    - authz.rs -> Authorization middleware. Usage '.layer(authz("permission"))'

  - routes/ -> Routes for the project.
    - admin/ -> Admin routes.
      - auth.rs -> create/delete/update/list auth tokens.

    - drive/ ->
      - google_drive.rs -> Google drive routes .
        - request_handler_get_primary_file_id -> Get primary file id in appdata folder.
        - request_handler_set_as_primary -> Create primary_file.json in appdata folder.
        - request_handler_get_logical_folders -> Get logical folders from appdata folder.
        - request_handler_get_secondary_drives -> Get secondary drives from appdata folder.
        - request_handler_get_appdata_file -> Get appdata file by file id.
        - request_handler_set_new_logical_folder -> Create new logical folder config file in appdata folder.
        - request_handler_set_new_secondary_drive -> Create new secondary_drive config file in appdata folder.

    - health/ -> uptime checks

    - profile/ -> Fetch user profile.
      - google.rs -> Fetch user profile from Google.
        - request_handler_user_info -> Fetch user profile from Google.

    - token/ -> Token routes.
      - google_web.rs -> Google web token routes.
        - request_handler_refresh_token -> Get refresh token from Google.
        - request_handler_access_token -> Get access token from Google using refresh token.

    - app_info.rs -> gives app status

  - allowed_clients.rs -> Auth keys loading logic. DO NOT EDIT.
  - constants.rs -> Constants.
  - error.rs -> AppError type and impls.
  - main.rs -> Entry point. DO NOT EDIT.
  - server.rs -> Server startup logic. DO NOT EDIT.
  - state.rs -> App state. DO NOT EDIT.
  - test_utils.rs -> Test utils. DO NOT EDIT.
  - validation.rs -> Validation logic. TO BE IMPLEMENTED LATER.
