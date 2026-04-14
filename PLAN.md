# Implementation Plan: Bot Access Control

## Overview
This document outlines the phased plan to introduce access control (allowed users) and group configurations to the `opencode-telegram-bot-rs` project. 

*Note: Based on current priorities, Phase 1 (Access Control) and the basic elements of Phase 2 (Group Logic) will be implemented now. Advanced group context features are deferred for later.*

## Phase 1: Basic Access Control (Immediate Execution)

### 1. Data Modeling & Configuration
* **User Identification**: We will use **Telegram User IDs** (`i64`) since they are strictly unique and immutable (unlike usernames).
* **Configuration Structure** (e.g., loaded from `.env` or a config file for now):
  ```rust
  struct AppConfig {
      // "*" translates to allowing everyone.
      // Otherwise, a restricted list of Telegram User IDs.
      pub allowed_users: AccessControl,
      // Message to send when an unauthorized user interacts.
      pub unauthorized_message: String,
  }

  enum AccessControl {
      All,
      Restricted(Vec<i64>),
  }
  ```

### 2. Onboarding & User Experience
* When an unauthorized user tries to use the bot (or in the documentation/setup guide), we need to instruct the admin/user on how to find their Telegram ID.
* We will recommend users forward a message to or interact with **`@userinfobot`** (or a similar standard bot) to easily retrieve their `i64` Telegram ID.

### 3. Access Middleware / Handler Logic
* Intercept incoming Telegram updates.
* **User Check**: Extract `message.from.id`. 
* If `allowed_users` is `Restricted`, verify the ID exists in the allowed list. 
* If the user is not authorized, send the `unauthorized_message` and drop the update so it doesn't reach the LLM/Agent logic.

---

## Phase 2: Group Configurations
*Extend the configuration to handle bot behavior in group chats.*

### Immediate Execution
* **Group Check**: Extract `message.chat.type`. If it's a group/supergroup:
  * **`allow_in_groups`**: This will be set to `true` by default.
  * **Mention Requirement**: The bot will *only* reply when it is explicitly tagged (e.g., `@our_bot_name` or reacting to a reply to its own message). Otherwise, it will stay completely silent.
  * **Stateless Group Responses**: It will only respond to the specific message it was tagged in, without maintaining context of other untagged messages in the group.

### Deferred (TODO)
* **Advanced Group Context**: We need to think about this in detail later (e.g., how to handle group message history, context tracking across multiple users in a single group, and whether it should read previous messages when tagged).


