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
      - mod.rs -> GoogleClient struct (holds shared reqwest::Client + GoogleConfig). Module declarations only.
      - config.rs -> GoogleConfig, AccessTokenPayload, RefreshTokenPayload, GenerateTokenPayload.
      - helpers.rs -> private filename encoding helpers: encode_email_for_filename, encode_drive_name, secondary_drive_filename, logical_folder_filename, timestamp_hex.
      - auth.rs -> OAuth + userinfo impl block: fetch_refresh_token, fetch_access_token, fetch_user_info.
      - appdata.rs -> appData folder impl block: list_appdata_files, get_primary_file_id, get_appdata_file, set_is_primary, get_secondary_drives, set_secondary_drive, get_logical_folders, set_logical_folder, get_appdata_file_by_name, set_appdata_file_by_name.
      - drive.rs -> Drive API proxy impl block: drive_about, list_drive_children, get_file_metadata_rich, create_drive_folder, upload_drive_file, delete_drive_item, copy_drive_item, download_drive_file_bytes.

  - logger/ -> Logger for the project. DO NOT EDIT.

  - middlewares/
    - authz.rs -> Authorization middleware. Usage '.layer(authz("permission"))'

  - routes/ -> Routes for the project.
    - admin/ -> Admin routes.
      - auth.rs -> create/delete/update/list auth tokens.

    - drive/ ->
      - mod.rs -> nests /google-drive; merges appdata and proxy sub-routers.
      - appdata.rs -> appData route handlers:
        - request_handler_get_primary_file_id -> Get primary file id in appdata folder.
        - request_handler_set_as_primary -> Create primary_file.json in appdata folder.
        - request_handler_get_secondary_drives -> Get secondary drives from appdata folder.
        - request_handler_get_appdata_file -> Get appdata file by file id.
        - request_handler_set_new_secondary_drive -> Create new secondary_drive config file in appdata folder.
        - request_handler_get_logical_folders -> Get logical folders from appdata folder.
        - request_handler_set_new_logical_folder -> Create new logical folder config file in appdata folder.
        - request_handler_get_appdata_file_by_name -> Get appdata file content by exact filename.
        - request_handler_set_appdata_file_by_name -> Upsert appdata file by exact filename.
      - proxy.rs -> Drive API proxy route handlers:
        - request_handler_drive_about -> Get Drive storage quota info.
        - request_handler_list_children -> List files/folders inside a Drive folder.
        - request_handler_file_metadata -> Get rich metadata for a single Drive item.
        - request_handler_create_folder -> Create a Drive folder.
        - request_handler_upload_file -> Multipart upload of a file to Drive.
        - request_handler_delete_item -> Delete a Drive file or folder.
        - request_handler_copy_item -> Copy a Drive item to a destination folder.
        - request_handler_download_file -> Download raw bytes of a Drive file.

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

  - openapi.json -> OpenAPI specification for the project. EDIT THIS FILE TO UPDATE THE API SPECIFICATION.