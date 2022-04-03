#[cfg(test)]
mod tests {
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, TokenDetails, TokenDetailsResponse,
    };
    use crate::state::Config;
    use cosmwasm_std::{coins, to_binary, Addr, Coin, Empty, Uint128};
    use cw20::Cw20Coin;
    use cw_multi_test::{
        next_block, App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor,
    };
    use whoami::msg::SurchargeInfo;

    pub fn contract_whoami_paths() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_whoami() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            whoami::entry::execute,
            whoami::entry::instantiate,
            whoami::entry::query,
        );
        Box::new(contract)
    }

    pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1000000000),
                    }],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1000000000),
                    }],
                )
                .unwrap();
        })
    }

    fn instantiate_cw20(app: &mut App) -> Addr {
        let cw20_id = app.store_code(contract_cw20());
        let msg = cw20_base::msg::InstantiateMsg {
            name: "Token".to_string(),
            symbol: "TOK".to_string(),
            decimals: 6,
            initial_balances: vec![
                Cw20Coin {
                    address: ADMIN.to_string(),
                    amount: Uint128::new(10000000),
                },
                Cw20Coin {
                    address: USER.to_string(),
                    amount: Uint128::new(10000000),
                },
            ],
            mint: None,
            marketing: None,
        };
        app.instantiate_contract(cw20_id, Addr::unchecked(ADMIN), &msg, &[], "cw20", None)
            .unwrap()
    }

    fn instantiate_whoami(app: &mut App) -> Addr {
        let whoami_id = app.store_code(contract_whoami());
        let msg = whoami::msg::InstantiateMsg {
            name: "Decentralized Name Service".to_string(),
            symbol: "WHO".to_string(),
            native_denom: NATIVE_DENOM.to_string(),
            native_decimals: 6,
            token_cap: None,
            base_mint_fee: Some(Uint128::new(1000000)),
            burn_percentage: Some(50),
            short_name_surcharge: Some(SurchargeInfo {
                surcharge_fee: Uint128::new(1000000),
                surcharge_max_characters: 5,
            }),
            admin_address: ADMIN.to_string(),
            username_length_cap: Some(20),
        };

        app.instantiate_contract(whoami_id, Addr::unchecked(ADMIN), &msg, &[], "whoami", None)
            .unwrap()
    }

    fn instantiate_whoami_paths(
        app: &mut App,
        whoami_addr: Addr,
        token_details: Option<TokenDetails>,
    ) -> Addr {
        let whoami_paths = app.store_code(contract_whoami_paths());
        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            whoami_address: whoami_addr.to_string(),
            token_details,
        };
        app.instantiate_contract(
            whoami_paths,
            Addr::unchecked(ADMIN),
            &msg,
            &[],
            "whoami-paths",
            None,
        )
        .unwrap()
    }

    fn setup_test_case(app: &mut App, with_token: bool) -> (Addr, Addr, Addr) {
        let cw20_addr = instantiate_cw20(app);
        let whoami_addr = instantiate_whoami(app);
        app.update_block(next_block);
        let paths_addr = if with_token {
            instantiate_whoami_paths(
                app,
                whoami_addr.clone(),
                Some(TokenDetails {
                    token_address: cw20_addr.to_string(),
                    token_cost: Uint128::new(100),
                }),
            )
        } else {
            instantiate_whoami_paths(app, whoami_addr.clone(), None)
        };
        (cw20_addr, whoami_addr, paths_addr)
    }

    fn mint_name(
        app: &mut App,
        whoami_addr: Addr,
        sender: &str,
        name: &str,
    ) -> anyhow::Result<AppResponse> {
        let msg = whoami::ExecuteMsg::Mint(whoami::msg::MintMsg {
            token_id: name.to_string(),
            owner: sender.to_string(),
            token_uri: None,
            extension: whoami::msg::Extension {
                image: None,
                image_data: None,
                email: None,
                external_url: None,
                public_name: None,
                public_bio: None,
                twitter_id: None,
                discord_id: None,
                telegram_id: None,
                keybase_id: None,
                validator_operator_address: None,
                contract_address: None,
                parent_token_id: None,
                pgp_public_key: None,
            },
        });
        app.execute_contract(
            Addr::unchecked(sender),
            whoami_addr,
            &msg,
            &coins(1000000, NATIVE_DENOM),
        )
    }

    fn transfer_name(
        app: &mut App,
        whoami_addr: Addr,
        sender: &str,
        to: String,
        token_id: String,
    ) -> anyhow::Result<AppResponse> {
        let msg = whoami::ExecuteMsg::SendNft {
            contract: to,
            token_id,
            msg: Default::default(),
        };
        app.execute_contract(Addr::unchecked(sender), whoami_addr, &msg, &[])
    }

    fn mint_path_no_tokens(
        app: &mut App,
        paths_addr: Addr,
        path: &str,
        sender: &str,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::MintPath {
            path: path.to_string(),
        };
        app.execute_contract(Addr::unchecked(sender), paths_addr, &msg, &[])
    }

    fn mint_path_tokens(
        app: &mut App,
        paths_addr: Addr,
        cw20_addr: Addr,
        path: &str,
        sender: &str,
        amount: u128,
    ) -> anyhow::Result<AppResponse> {
        let msg = cw20_base::msg::ExecuteMsg::Send {
            contract: paths_addr.to_string(),
            amount: Uint128::new(amount),
            msg: to_binary(&ReceiveMsg::MintPath {
                path: path.to_string(),
            })
            .unwrap(),
        };
        app.execute_contract(Addr::unchecked(sender), cw20_addr, &msg, &[])
    }

    #[test]
    fn test_instantiate() {
        let mut app = mock_app();
        let (_cw20, _whoami, _paths) = setup_test_case(&mut app, false);
        let (_cw20, _whoami, _paths) = setup_test_case(&mut app, true);
    }

    #[test]
    fn test_receive_nft() {
        let mut app = mock_app();
        let (_cw20, whoami, paths) = setup_test_case(&mut app, false);
        let name = "howl_base";
        mint_name(&mut app, whoami.clone(), ADMIN, name).unwrap();
        let name2 = "howl_base2";
        mint_name(&mut app, whoami.clone(), ADMIN, name2).unwrap();
        let user_name = "user_name";
        mint_name(&mut app, whoami.clone(), USER, user_name).unwrap();

        // Cannot transfer from non admin
        transfer_name(
            &mut app,
            whoami.clone(),
            USER,
            paths.to_string(),
            user_name.to_string(),
        )
        .unwrap_err();

        transfer_name(
            &mut app,
            whoami.clone(),
            ADMIN,
            paths.to_string(),
            name.to_string(),
        )
        .unwrap();

        let msg = QueryMsg::Config {};
        let config: Config = app.wrap().query_wasm_smart(paths.clone(), &msg).unwrap();
        assert_eq!(config.token_id, Some(name.to_string()));

        // Cannot transfer another as already transferred
        transfer_name(
            &mut app,
            whoami,
            ADMIN,
            paths.to_string(),
            name2.to_string(),
        )
        .unwrap_err();
    }

    #[test]
    fn test_paths_no_cost() {
        // Setup
        let mut app = mock_app();
        let (cw20, whoami, paths) = setup_test_case(&mut app, false);
        let name = "howl_base";
        mint_name(&mut app, whoami.clone(), ADMIN, name).unwrap();
        // Expect error no name to mint off of
        mint_path_no_tokens(&mut app, paths.clone(), "a", USER).unwrap_err();

        // Give the contract the name
        transfer_name(&mut app, whoami, ADMIN, paths.to_string(), name.to_string()).unwrap();

        // Success
        mint_path_no_tokens(&mut app, paths.clone(), "a", USER).unwrap();
        mint_path_no_tokens(&mut app, paths.clone(), "b", ADMIN).unwrap();

        // Expect error already minted path
        mint_path_no_tokens(&mut app, paths.clone(), "b", USER).unwrap_err();

        // Expect error you do not need to pay
        mint_path_tokens(&mut app, paths, cw20, "c", USER, 1000).unwrap_err();
    }

    #[test]
    fn test_paths_cost() {
        let mut app = mock_app();
        let (cw20, whoami, paths) = setup_test_case(&mut app, true);
        let name = "howl_base";
        mint_name(&mut app, whoami.clone(), ADMIN, name).unwrap();
        // Expect error no name to mint off of
        mint_path_tokens(&mut app, paths.clone(), cw20.clone(), "a", USER, 1000).unwrap_err();

        // Give the contract the name
        transfer_name(&mut app, whoami, ADMIN, paths.to_string(), name.to_string()).unwrap();

        // Success
        mint_path_tokens(&mut app, paths.clone(), cw20.clone(), "a", USER, 100).unwrap();
        mint_path_tokens(&mut app, paths.clone(), cw20.clone(), "b", ADMIN, 100).unwrap();

        // Expect error not enough paid
        mint_path_tokens(&mut app, paths.clone(), cw20.clone(), "d", ADMIN, 99).unwrap_err();

        // Expect error already minted path
        mint_path_tokens(&mut app, paths.clone(), cw20, "b", USER, 100).unwrap_err();

        // Expect error you need to pay
        mint_path_no_tokens(&mut app, paths, "c", USER).unwrap_err();
    }

    #[test]
    fn test_update_admin() {
        let mut app = mock_app();
        let (_cw20, _whoami, paths) = setup_test_case(&mut app, true);

        // Set admin to USER
        let msg = ExecuteMsg::UpdateAdmin {
            new_admin: USER.to_string(),
        };

        // Fails as USER is not the admin
        app.execute_contract(Addr::unchecked(USER), paths.clone(), &msg, &[])
            .unwrap_err();

        // Success
        app.execute_contract(Addr::unchecked(ADMIN), paths.clone(), &msg, &[])
            .unwrap();

        let msg = QueryMsg::Config {};
        let config: Config = app.wrap().query_wasm_smart(paths, &msg).unwrap();
        assert_eq!(config.admin, Addr::unchecked(USER));
    }

    #[test]
    fn test_update_token_details() {
        let mut app = mock_app();
        let (cw20, _whoami, paths) = setup_test_case(&mut app, false);

        // No token
        let msg = QueryMsg::TokenDetails {};
        let config: TokenDetailsResponse =
            app.wrap().query_wasm_smart(paths.clone(), &msg).unwrap();
        assert_eq!(config.token_details, None);

        let msg = ExecuteMsg::UpdateTokenDetails {
            new_token_details: Some(TokenDetails {
                token_address: cw20.to_string(),
                token_cost: Uint128::new(50),
            }),
        };

        // Fails as USER is not the admin
        app.execute_contract(Addr::unchecked(USER), paths.clone(), &msg, &[])
            .unwrap_err();

        // Success
        app.execute_contract(Addr::unchecked(ADMIN), paths.clone(), &msg, &[])
            .unwrap();

        // Now there is a token
        let msg = QueryMsg::TokenDetails {};
        let config: TokenDetailsResponse =
            app.wrap().query_wasm_smart(paths.clone(), &msg).unwrap();
        assert_eq!(
            config.token_details,
            Some(TokenDetails {
                token_address: cw20.to_string(),
                token_cost: Uint128::new(50)
            })
        );

        // Remove the token
        let msg = ExecuteMsg::UpdateTokenDetails {
            new_token_details: None,
        };

        app.execute_contract(Addr::unchecked(ADMIN), paths.clone(), &msg, &[])
            .unwrap();

        // No token again
        let msg = QueryMsg::TokenDetails {};
        let config: TokenDetailsResponse = app.wrap().query_wasm_smart(paths, &msg).unwrap();
        assert_eq!(config.token_details, None);
    }
}
