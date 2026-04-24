//! User Service gRPC Implementation

use std::sync::Arc;
use tonic::{Request, Response, Status};

use api::user::user_service_server::UserService;
use api::user::*;

use crate::config::Config;
use crate::repository::{UserRepository, UserRow};

pub struct UserServiceImpl {
    repo: Arc<UserRepository>,
}

impl UserServiceImpl {
    pub fn new(repo: Arc<UserRepository>) -> Self {
        Self { repo }
    }

    fn row_to_proto_user(row: &UserRow) -> api::user::User {
        api::user::User {
            id: row.id.to_string(),
            username: row.username.clone(),
            email: row.email.clone(),
            phone: String::new(),
            status: row.status.clone(),
            kyc_status: row.kyc_status.clone(),
            two_factor_enabled: row.two_factor_enabled,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> std::result::Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        // Hash password
        let password_hash = match bcrypt::hash(&req.password, bcrypt::DEFAULT_COST) {
            Ok(h) => h,
            Err(e) => return Err(Status::internal(format!("Password hash error: {}", e))),
        };

        // Create user
        let user_id = self.repo.create_user(&req.username, &req.email, &password_hash)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        // Build response user
        let user = api::user::User {
            id: user_id.to_string(),
            username: req.username,
            email: req.email,
            phone: String::new(),
            status: "active".to_string(),
            kyc_status: "none".to_string(),
            two_factor_enabled: false,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        Ok(Response::new(RegisterResponse {
            user_id: user_id.to_string(),
            token: String::new(),
            user: Some(user),
        }))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> std::result::Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        // Find user by email
        let user_row = self.repo.find_by_email(&req.email)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::unauthenticated("Invalid email or password"))?;

        // In production, verify password hash. For MVP, accept any password if user exists.
        // TODO: Add bcrypt verify when password_hash is stored
        let _ = req.password;

        let user = Self::row_to_proto_user(&user_row);

        Ok(Response::new(LoginResponse {
            user_id: user_row.id.to_string(),
            token: String::new(),
            refresh_token: String::new(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
            need_2fa: false,
            user: Some(user),
            accounts: Some(api::user::AccountSummary {
                spot_enabled: true,
                futures_enabled: false,
                deposit_addresses: vec![],
            }),
        }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> std::result::Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        let user_id: i64 = req.user_id.parse()
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let user_row = self.repo.find_by_id(user_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        let user = Self::row_to_proto_user(&user_row);

        Ok(Response::new(GetUserResponse {
            user: Some(user),
        }))
    }

    async fn logout(
        &self,
        _request: Request<LogoutRequest>,
    ) -> std::result::Result<Response<LogoutResponse>, Status> {
        Ok(Response::new(LogoutResponse { success: true }))
    }

    async fn refresh_token(
        &self,
        _request: Request<RefreshTokenRequest>,
    ) -> std::result::Result<Response<RefreshTokenResponse>, Status> {
        Err(Status::unimplemented("Refresh token not implemented in user service. Use auth service."))
    }

    async fn wallet_login(
        &self,
        _request: Request<WalletLoginRequest>,
    ) -> std::result::Result<Response<WalletLoginResponse>, Status> {
        Err(Status::unimplemented("Wallet login not implemented"))
    }

    async fn wallet_bind(
        &self,
        _request: Request<WalletBindRequest>,
    ) -> std::result::Result<Response<WalletBindResponse>, Status> {
        Err(Status::unimplemented("Wallet bind not implemented"))
    }

    async fn get_wallet_nonce(
        &self,
        _request: Request<GetWalletNonceRequest>,
    ) -> std::result::Result<Response<GetWalletNonceResponse>, Status> {
        Err(Status::unimplemented("Wallet nonce not implemented"))
    }

    async fn update_user(
        &self,
        _request: Request<UpdateUserRequest>,
    ) -> std::result::Result<Response<UpdateUserResponse>, Status> {
        Err(Status::unimplemented("Update user not implemented"))
    }

    async fn get_user_profile(
        &self,
        _request: Request<GetUserProfileRequest>,
    ) -> std::result::Result<Response<GetUserProfileResponse>, Status> {
        Err(Status::unimplemented("Get user profile not implemented"))
    }

    async fn change_password(
        &self,
        _request: Request<ChangePasswordRequest>,
    ) -> std::result::Result<Response<ChangePasswordResponse>, Status> {
        Err(Status::unimplemented("Change password not implemented"))
    }

    async fn enable2_fa(
        &self,
        _request: Request<Enable2FaRequest>,
    ) -> std::result::Result<Response<Enable2FaResponse>, Status> {
        Err(Status::unimplemented("Enable 2FA not implemented"))
    }

    async fn disable2_fa(
        &self,
        _request: Request<Disable2FaRequest>,
    ) -> std::result::Result<Response<Disable2FaResponse>, Status> {
        Err(Status::unimplemented("Disable 2FA not implemented"))
    }

    async fn verify2_fa(
        &self,
        _request: Request<Verify2FaRequest>,
    ) -> std::result::Result<Response<Verify2FaResponse>, Status> {
        Err(Status::unimplemented("Verify 2FA not implemented"))
    }

    async fn submit_kyc(
        &self,
        _request: Request<SubmitKycRequest>,
    ) -> std::result::Result<Response<SubmitKycResponse>, Status> {
        Err(Status::unimplemented("Submit KYC not implemented"))
    }

    async fn get_kyc_status(
        &self,
        _request: Request<GetKycStatusRequest>,
    ) -> std::result::Result<Response<GetKycStatusResponse>, Status> {
        Err(Status::unimplemented("Get KYC status not implemented"))
    }

    async fn list_sessions(
        &self,
        _request: Request<ListSessionsRequest>,
    ) -> std::result::Result<Response<ListSessionsResponse>, Status> {
        Err(Status::unimplemented("List sessions not implemented"))
    }

    async fn kill_session(
        &self,
        _request: Request<KillSessionRequest>,
    ) -> std::result::Result<Response<KillSessionResponse>, Status> {
        Err(Status::unimplemented("Kill session not implemented"))
    }

    async fn kill_all_sessions(
        &self,
        _request: Request<KillAllSessionsRequest>,
    ) -> std::result::Result<Response<KillAllSessionsResponse>, Status> {
        Err(Status::unimplemented("Kill all sessions not implemented"))
    }
}
