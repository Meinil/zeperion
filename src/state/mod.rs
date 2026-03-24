use chrono::{DateTime, Utc};

use crate::domain::{
    account::{Account, generate_unique_offline_uuid_for_skin_reference},
    background::BackgroundImage,
    download::{DownloadFilters, DownloadTaskRecord, RemoteGameVersion},
    game_version::InstalledGameVersion,
    java::JavaRuntime,
    settings::{AppSettings, MirrorSource},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddOfflineAccountError {
    EmptyUsername,
    DuplicateUsername,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppMessage {
    pub id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub is_read: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppStore {
    pub settings: AppSettings,
    pub accounts: Vec<Account>,
    pub java_runtimes: Vec<JavaRuntime>,
    pub versions: Vec<InstalledGameVersion>,
    pub remote_versions: Vec<RemoteGameVersion>,
    pub backgrounds: Vec<BackgroundImage>,
    pub custom_mirrors: Vec<MirrorSource>,
    pub download_tasks: Vec<DownloadTaskRecord>,
    pub selected_version_id: Option<String>,
    pub manifest_loading: bool,
    pub manifest_error: Option<String>,
    pub download_filters: DownloadFilters,
    pub pending_account_delete_id: Option<String>,
    pub notification: Option<String>,
    pub messages: Vec<AppMessage>,
    pub message_drawer_open: bool,
    pub active_launch_id: Option<String>,
}

impl Default for AppStore {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
            accounts: Vec::new(),
            java_runtimes: Vec::new(),
            versions: Vec::new(),
            remote_versions: Vec::new(),
            backgrounds: Vec::new(),
            custom_mirrors: Vec::new(),
            download_tasks: Vec::new(),
            selected_version_id: None,
            manifest_loading: false,
            manifest_error: None,
            download_filters: DownloadFilters::default(),
            pending_account_delete_id: None,
            notification: None,
            messages: Vec::new(),
            message_drawer_open: false,
            active_launch_id: None,
        }
    }
}

impl AppStore {
    pub fn from_bootstrap(
        settings: AppSettings,
        accounts: Vec<Account>,
        java_runtimes: Vec<JavaRuntime>,
        versions: Vec<InstalledGameVersion>,
        backgrounds: Vec<BackgroundImage>,
        mirrors: Vec<MirrorSource>,
        download_tasks: Vec<DownloadTaskRecord>,
        messages: Vec<AppMessage>,
    ) -> Self {
        let selected_version_id = versions
            .iter()
            .find(|version| version.is_downloaded)
            .map(|version| version.id.clone());

        Self {
            settings,
            accounts,
            java_runtimes,
            versions,
            backgrounds,
            custom_mirrors: mirrors,
            download_tasks,
            messages,
            selected_version_id,
            ..Self::default()
        }
    }

    pub fn selected_account(&self) -> Option<&Account> {
        self.accounts.iter().find(|account| account.selected)
    }

    pub fn selected_version(&self) -> Option<&InstalledGameVersion> {
        self.selected_version_id
            .as_ref()
            .and_then(|id| self.versions.iter().find(|version| &version.id == id))
    }

    pub fn default_java(&self) -> Option<&JavaRuntime> {
        self.java_runtimes.iter().find(|runtime| runtime.is_default)
    }

    pub fn add_offline_account(
        &mut self,
        username: String,
    ) -> Result<Account, AddOfflineAccountError> {
        self.add_offline_account_with_options(username, None, None)
    }

    pub fn add_offline_account_with_options(
        &mut self,
        username: String,
        provided_uuid: Option<String>,
        skin_reference: Option<String>,
    ) -> Result<Account, AddOfflineAccountError> {
        let normalized_username = username.trim().to_string();
        if normalized_username.is_empty() {
            return Err(AddOfflineAccountError::EmptyUsername);
        }
        if self.accounts.iter().any(|account| {
            account
                .username
                .trim()
                .eq_ignore_ascii_case(normalized_username.as_str())
        }) {
            return Err(AddOfflineAccountError::DuplicateUsername);
        }

        for account in &mut self.accounts {
            account.selected = false;
        }

        let resolved_reference = skin_reference
            .clone()
            .unwrap_or_else(|| "random".to_string());
        let resolved_uuid = if let Some(uuid) = provided_uuid {
            if self.accounts.iter().any(|account| account.uuid == uuid) {
                generate_unique_offline_uuid_for_skin_reference(&resolved_reference, |candidate| {
                    self.accounts
                        .iter()
                        .any(|account| account.uuid == candidate)
                })
            } else {
                uuid
            }
        } else {
            generate_unique_offline_uuid_for_skin_reference(&resolved_reference, |candidate| {
                self.accounts
                    .iter()
                    .any(|account| account.uuid == candidate)
            })
        };

        let mut account =
            Account::offline_with_options(normalized_username, Some(resolved_uuid), skin_reference);
        account.selected = true;
        self.accounts.push(account.clone());
        self.pending_account_delete_id = None;
        Ok(account)
    }

    pub fn upsert_account(&mut self, mut account: Account) -> Account {
        if account.selected {
            for existing in &mut self.accounts {
                existing.selected = false;
            }
        }

        if let Some(existing) = self.accounts.iter_mut().find(|item| item.id == account.id) {
            account.selected = account.selected || existing.selected;
            *existing = account.clone();
        } else {
            self.accounts.push(account.clone());
        }

        self.pending_account_delete_id = None;
        self.accounts.sort_by_key(|item| !item.selected);
        account
    }

    pub fn select_account(&mut self, account_id: &str) -> bool {
        if self
            .accounts
            .iter()
            .find(|account| account.id == account_id)
            .map(|account| account.selected)
            .unwrap_or(false)
        {
            return false;
        }

        for account in &mut self.accounts {
            account.selected = account.id == account_id;
        }
        true
    }

    pub fn replace_java_runtimes(&mut self, runtimes: Vec<JavaRuntime>) {
        self.java_runtimes = runtimes;
    }

    pub fn set_default_java(&mut self, runtime_id: &str) -> bool {
        if self
            .java_runtimes
            .iter()
            .find(|runtime| runtime.id == runtime_id)
            .map(|runtime| runtime.is_default)
            .unwrap_or(false)
        {
            return false;
        }

        for runtime in &mut self.java_runtimes {
            runtime.is_default = runtime.id == runtime_id;
        }
        true
    }

    pub fn set_selected_version_id(&mut self, version_id: String) -> bool {
        if self.selected_version_id.as_deref() == Some(version_id.as_str()) {
            return false;
        }

        self.selected_version_id = Some(version_id);
        true
    }

    pub fn replace_remote_versions(&mut self, versions: Vec<RemoteGameVersion>) {
        self.remote_versions = versions;
        self.manifest_error = None;
        self.manifest_loading = false;
    }

    pub fn set_manifest_loading(&mut self, loading: bool) {
        self.manifest_loading = loading;
        if loading {
            self.manifest_error = None;
        }
    }

    pub fn set_manifest_error(&mut self, error: String) {
        self.manifest_error = Some(error);
        self.manifest_loading = false;
    }

    pub fn is_installed_version(&self, version_id: &str) -> bool {
        self.versions
            .iter()
            .any(|version| version.version_name == version_id && version.is_downloaded)
    }

    pub fn installed_version_by_name(&self, version_name: &str) -> Option<&InstalledGameVersion> {
        self.versions
            .iter()
            .find(|version| version.version_name == version_name)
    }

    pub fn upsert_installed_version(&mut self, incoming: InstalledGameVersion) {
        if let Some(existing) = self
            .versions
            .iter_mut()
            .find(|version| version.version_name == incoming.version_name)
        {
            *existing = incoming;
        } else {
            self.versions.push(incoming);
            self.versions
                .sort_by(|left, right| right.release_time.cmp(&left.release_time));
        }
    }

    pub fn request_delete_account(&mut self, account_id: String) {
        self.pending_account_delete_id = Some(account_id);
    }

    pub fn push_message(&mut self, content: String) -> AppMessage {
        let id = format!("msg-{}", Utc::now().timestamp_micros());
        let message = AppMessage {
            id,
            content,
            created_at: Utc::now(),
            is_read: false,
        };
        self.messages.insert(0, message.clone());
        message
    }

    pub fn remove_message(&mut self, message_id: &str) {
        self.messages.retain(|message| message.id != message_id);
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    pub fn mark_message_read(&mut self, message_id: &str) -> bool {
        if let Some(message) = self
            .messages
            .iter_mut()
            .find(|message| message.id == message_id)
        {
            if message.is_read {
                return false;
            }
            message.is_read = true;
            return true;
        }
        false
    }

    pub fn unread_message_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|message| !message.is_read)
            .count()
    }

    pub fn set_active_launch(&mut self, launch_id: String) {
        self.active_launch_id = Some(launch_id);
    }

    pub fn clear_active_launch(&mut self) {
        self.active_launch_id = None;
    }

    pub fn is_launch_running(&self) -> bool {
        self.active_launch_id.is_some()
    }

    pub fn cancel_delete_account(&mut self) {
        self.pending_account_delete_id = None;
    }

    pub fn remove_account(&mut self, account_id: &str) {
        self.accounts.retain(|account| account.id != account_id);
        self.pending_account_delete_id = None;
        if self.accounts.iter().all(|account| !account.selected) {
            if let Some(first) = self.accounts.first_mut() {
                first.selected = true;
            }
        }
    }

    pub fn add_background(&mut self, background: BackgroundImage) {
        if background.is_active {
            for item in &mut self.backgrounds {
                item.is_active = false;
            }
        }
        self.backgrounds.insert(0, background);
    }

    pub fn set_active_background(&mut self, background_id: &str) -> bool {
        if self
            .backgrounds
            .iter()
            .find(|item| item.id == background_id)
            .map(|item| item.is_active)
            .unwrap_or(false)
        {
            return false;
        }

        for item in &mut self.backgrounds {
            item.is_active = item.id == background_id;
        }
        true
    }

    pub fn remove_background(&mut self, background_id: &str) {
        self.backgrounds.retain(|item| item.id != background_id);
    }

    pub fn add_custom_mirror(&mut self, mirror: MirrorSource) {
        self.custom_mirrors.push(mirror);
        self.custom_mirrors.sort_by(|a, b| a.name.cmp(&b.name));
    }

    pub fn remove_custom_mirror(&mut self, mirror_id: &str) {
        self.custom_mirrors.retain(|mirror| mirror.id != mirror_id);
    }

    pub fn add_download_task(&mut self, task: DownloadTaskRecord) {
        if let Some(existing) = self
            .download_tasks
            .iter_mut()
            .find(|item| item.id == task.id)
        {
            *existing = task;
        } else {
            self.download_tasks.insert(0, task);
        }
        self.download_tasks
            .sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    }

    pub fn remove_version_by_name(&mut self, version_name: &str) {
        let removed_ids: Vec<String> = self
            .versions
            .iter()
            .filter(|version| version.version_name == version_name)
            .map(|version| version.id.clone())
            .collect();
        self.versions
            .retain(|version| version.version_name != version_name);
        self.download_tasks
            .retain(|task| task.package_id != version_name && task.package_name != version_name);
        if self
            .selected_version_id
            .as_ref()
            .map(|selected| removed_ids.iter().any(|id| id == selected))
            .unwrap_or(false)
        {
            self.selected_version_id = None;
        }
    }

    pub fn update_download_task_progress(
        &mut self,
        task_id: &str,
        status: crate::domain::download::DownloadStatus,
        downloaded_bytes: i64,
        total_bytes: Option<i64>,
    ) {
        if let Some(task) = self
            .download_tasks
            .iter_mut()
            .find(|task| task.id == task_id)
        {
            task.status = status;
            task.downloaded_bytes = downloaded_bytes;
            task.total_bytes = total_bytes;
            task.updated_at = chrono::Utc::now();
            self.download_tasks
                .sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        }
    }
}
