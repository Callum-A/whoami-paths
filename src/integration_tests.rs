#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, InstantiateMsg, PaymentDetails, QueryMsg, ReceiveMsg};
    use crate::state::Config;
    use cosmwasm_std::{coins, to_binary, Addr, Coin, Empty, Uint128};
    use cw20::{Cw20Coin, Cw20ExecuteMsg};
    use cw721::{Cw721QueryMsg, OwnerOfResponse};
    use cw_multi_test::{
        next_block, App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor,
    };
    use whoami::msg::SurchargeInfo;

    const USER: &str = "addr1";
    const ADMIN: &str = "addr2";
    const NATIVE_DENOM: &str = "ujunox";
    const INVALID_DENOM: &str = "uinvalid";

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

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(1000000000),
                        },
                        Coin {
                            denom: INVALID_DENOM.to_string(),
                            amount: Uint128::new(1000000000),
                        },
                    ],
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(1000000000),
                        },
                        Coin {
                            denom: INVALID_DENOM.to_string(),
                            amount: Uint128::new(1000000000),
                        },
                    ],
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
        payment_details: Option<PaymentDetails>,
    ) -> Addr {
        let whoami_paths = app.store_code(contract_whoami_paths());
        let msg = InstantiateMsg {
            admin: ADMIN.to_string(),
            whoami_address: whoami_addr.to_string(),
            payment_details,
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

    fn setup_test_case(app: &mut App, payment_details: Option<PaymentDetails>) -> (Addr, Addr) {
        let whoami_addr = instantiate_whoami(app);
        let paths_addr = instantiate_whoami_paths(app, whoami_addr.clone(), payment_details);
        app.update_block(next_block);
        (whoami_addr, paths_addr)
    }

    fn setup_test_case_with_name(
        app: &mut App,
        payment_details: Option<PaymentDetails>,
    ) -> (Addr, Addr, String) {
        let (whoami, paths) = setup_test_case(app, payment_details);

        // Mint the name
        let token_id = "root_name".to_string();
        mint_name(app, whoami.clone(), ADMIN, &token_id).unwrap();

        // Transfer to the contract
        transfer_name(
            app,
            whoami.clone(),
            ADMIN,
            paths.to_string(),
            token_id.clone(),
        )
        .unwrap();
        (whoami, paths, token_id)
    }

    fn mint_name(
        app: &mut App,
        whoami_addr: Addr,
        sender: &str,
        name: &str,
    ) -> anyhow::Result<AppResponse> {
        let msg = whoami::msg::ExecuteMsg::Mint(whoami::msg::MintMsg {
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
        let msg = whoami::msg::ExecuteMsg::SendNft {
            contract: to,
            token_id,
            msg: Default::default(),
        };
        app.execute_contract(Addr::unchecked(sender), whoami_addr, &msg, &[])
    }

    fn mint_path_native(
        app: &mut App,
        paths_addr: Addr,
        sender: &str,
        path: &str,
        payment: Vec<Coin>,
    ) -> anyhow::Result<AppResponse> {
        app.execute_contract(
            Addr::unchecked(sender),
            paths_addr,
            &ExecuteMsg::MintPath {
                path: path.to_string(),
            },
            &payment,
        )
    }

    fn mint_path_cw20(
        app: &mut App,
        cw20_addr: Addr,
        paths_addr: Addr,
        sender: &str,
        amount: Uint128,
        path: &str,
    ) -> anyhow::Result<AppResponse> {
        let msg = Cw20ExecuteMsg::Send {
            contract: paths_addr.to_string(),
            amount,
            msg: to_binary(&ReceiveMsg::MintPath {
                path: path.to_string(),
            })?,
        };
        app.execute_contract(Addr::unchecked(sender), cw20_addr, &msg, &[])
    }

    fn update_admin(
        app: &mut App,
        paths_addr: Addr,
        sender: &str,
        new_admin: String,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::UpdateAdmin { new_admin };
        app.execute_contract(Addr::unchecked(sender), paths_addr, &msg, &[])
    }

    fn withdraw_token(
        app: &mut App,
        paths_addr: Addr,
        sender: &str,
    ) -> anyhow::Result<AppResponse> {
        let msg = ExecuteMsg::WithdrawRootToken {};
        app.execute_contract(Addr::unchecked(sender), paths_addr, &msg, &[])
    }

    fn get_config(app: &mut App, paths_addr: Addr) -> Config {
        app.wrap()
            .query_wasm_smart(paths_addr, &QueryMsg::Config {})
            .unwrap()
    }

    fn get_nft_owner(app: &mut App, whoami_addr: Addr, token_id: String) -> OwnerOfResponse {
        let msg = Cw721QueryMsg::OwnerOf {
            token_id,
            include_expired: None,
        };
        app.wrap().query_wasm_smart(whoami_addr, &msg).unwrap()
    }

    #[test]
    fn test_instantiate_valid() {
        let mut app = mock_app();
        // Instantiate with no payment
        let (_whoami, _paths) = setup_test_case(&mut app, None);
        // Instantiate with valid cw20
        let cw20_addr = instantiate_cw20(&mut app);
        let (_whoami, _paths) = setup_test_case(
            &mut app,
            Some(PaymentDetails::Cw20 {
                token_address: cw20_addr.to_string(),
                amount: Uint128::new(100),
            }),
        );
        // Instantiate with native
        let (_whoami, _paths) = setup_test_case(
            &mut app,
            Some(PaymentDetails::Native {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::new(100),
            }),
        );
    }

    #[test]
    #[should_panic(expected = "The token address provided is not a valid CW20 token")]
    fn test_instantiate_invalid_cw20() {
        let mut app = mock_app();
        // Instantiate with non CW20 addr
        let (_whoami, _paths) = setup_test_case(
            &mut app,
            Some(PaymentDetails::Cw20 {
                token_address: USER.to_string(),
                amount: Uint128::new(100),
            }),
        );
    }

    #[test]
    #[should_panic(expected = "You have specified payment details but amount is set to 0")]
    fn test_instantiate_invalid_cw20_amount() {
        let mut app = mock_app();
        let cw20_addr = instantiate_cw20(&mut app);
        // Instantiate with 0 amount
        let (_whoami, _paths) = setup_test_case(
            &mut app,
            Some(PaymentDetails::Cw20 {
                token_address: cw20_addr.to_string(),
                amount: Uint128::zero(),
            }),
        );
    }

    #[test]
    #[should_panic(expected = "You have specified payment details but amount is set to 0")]
    fn test_instantiate_invalid_native_amount() {
        let mut app = mock_app();
        // Instantiate with 0 amount
        let (_whoami, _paths) = setup_test_case(
            &mut app,
            Some(PaymentDetails::Native {
                denom: NATIVE_DENOM.to_string(),
                amount: Uint128::zero(),
            }),
        );
    }

    #[test]
    fn test_receive_root_name() {
        let mut app = mock_app();
        let (whoami, paths) = setup_test_case(&mut app, None);

        // Check config, name is None
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.token_id, None);

        // Mint the name
        let token_id = "root_name".to_string();
        mint_name(&mut app, whoami.clone(), ADMIN, &token_id).unwrap();

        // Transfer to the contract
        transfer_name(&mut app, whoami, ADMIN, paths.to_string(), token_id.clone()).unwrap();

        // Check config, name is Some("root_name")
        let config = get_config(&mut app, paths);
        assert_eq!(config.token_id, Some(token_id));
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_receive_root_name_invalid_nft_contract() {
        let mut app = mock_app();
        let (_whoami, paths) = setup_test_case(&mut app, None);
        // Create again to get a different nft contract, it is invalid
        let (whoami_invalid, _paths) = setup_test_case(&mut app, None);

        // Check config, name is None
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.token_id, None);

        // Mint the name
        let token_id = "root_name".to_string();
        mint_name(&mut app, whoami_invalid.clone(), ADMIN, &token_id).unwrap();

        // Transfer to the contract
        transfer_name(
            &mut app,
            whoami_invalid,
            ADMIN,
            paths.to_string(),
            token_id.clone(),
        )
        .unwrap();

        // Check config, name is None
        let config = get_config(&mut app, paths);
        assert_eq!(config.token_id, None);
    }

    #[test]
    #[should_panic(expected = "The root token has already been set")]
    fn test_receive_root_name_root_name_already_set() {
        let mut app = mock_app();
        let (whoami, paths) = setup_test_case(&mut app, None);

        // Check config, name is None
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.token_id, None);

        // Mint the name
        let token_id = "root_name".to_string();
        mint_name(&mut app, whoami.clone(), ADMIN, &token_id).unwrap();

        // Mint a second name
        let token_id_invalid = "already_set".to_string();
        mint_name(&mut app, whoami.clone(), ADMIN, &token_id_invalid).unwrap();

        // Transfer to the contract
        transfer_name(
            &mut app,
            whoami.clone(),
            ADMIN,
            paths.to_string(),
            token_id.clone(),
        )
        .unwrap();

        // Check config, name is Some("root_name")
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.token_id, Some(token_id));

        // Try to transfer to the contract
        transfer_name(
            &mut app,
            whoami,
            ADMIN,
            paths.to_string(),
            token_id_invalid.clone(),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_receive_root_name_non_admin() {
        let mut app = mock_app();
        let (whoami, paths) = setup_test_case(&mut app, None);

        // Check config, name is None
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.token_id, None);

        // Mint the name
        let token_id = "root_name".to_string();
        mint_name(&mut app, whoami.clone(), USER, &token_id).unwrap();

        // Transfer to the contract
        transfer_name(&mut app, whoami, USER, paths.to_string(), token_id.clone()).unwrap();
    }

    #[test]
    fn test_update_admin() {
        let mut app = mock_app();
        let (_whoami, paths) = setup_test_case(&mut app, None);

        // Check config, admin is ADMIN
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.admin, Addr::unchecked(ADMIN));

        update_admin(&mut app, paths.clone(), ADMIN, USER.to_string()).unwrap();

        // Check config, admin is USER
        let config = get_config(&mut app, paths);
        assert_eq!(config.admin, Addr::unchecked(USER));
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_update_admin_invalid() {
        let mut app = mock_app();
        let (_whoami, paths) = setup_test_case(&mut app, None);

        // Check config, admin is ADMIN
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.admin, Addr::unchecked(ADMIN));

        update_admin(&mut app, paths, USER, USER.to_string()).unwrap();
    }

    #[test]
    fn test_withdraw_root_name() {
        let mut app = mock_app();
        let (whoami, paths) = setup_test_case(&mut app, None);

        // Mint the name
        let token_id = "root_name".to_string();
        mint_name(&mut app, whoami.clone(), ADMIN, &token_id).unwrap();

        // Transfer to the contract
        transfer_name(
            &mut app,
            whoami.clone(),
            ADMIN,
            paths.to_string(),
            token_id.clone(),
        )
        .unwrap();

        let resp = get_nft_owner(&mut app, whoami.clone(), token_id.clone());
        assert_eq!(resp.owner, paths.to_string());

        // Check config, name is Some("root_name")
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.token_id, Some(token_id.clone()));

        withdraw_token(&mut app, paths, ADMIN).unwrap();

        let resp = get_nft_owner(&mut app, whoami, token_id);
        assert_eq!(resp.owner, ADMIN.to_string());
    }

    #[test]
    #[should_panic(expected = "The root token has not been received yet")]
    fn test_withdraw_root_name_no_name() {
        let mut app = mock_app();
        let (_whoami, paths) = setup_test_case(&mut app, None);

        withdraw_token(&mut app, paths, ADMIN).unwrap();
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_withdraw_root_name_non_admin() {
        let mut app = mock_app();
        let (whoami, paths) = setup_test_case(&mut app, None);

        // Mint the name
        let token_id = "root_name".to_string();
        mint_name(&mut app, whoami.clone(), ADMIN, &token_id).unwrap();

        // Transfer to the contract
        transfer_name(
            &mut app,
            whoami.clone(),
            ADMIN,
            paths.to_string(),
            token_id.clone(),
        )
        .unwrap();

        let resp = get_nft_owner(&mut app, whoami, token_id.clone());
        assert_eq!(resp.owner, paths.to_string());

        // Check config, name is Some("root_name")
        let config = get_config(&mut app, paths.clone());
        assert_eq!(config.token_id, Some(token_id.clone()));

        withdraw_token(&mut app, paths, USER).unwrap();
    }

    mod native_payment {
        use crate::integration_tests::tests::{
            get_nft_owner, instantiate_cw20, mint_path_cw20, mint_path_native, mock_app,
            setup_test_case, setup_test_case_with_name, INVALID_DENOM, NATIVE_DENOM, USER,
        };
        use crate::msg::PaymentDetails;
        use cosmwasm_std::{coins, Uint128};

        #[test]
        fn test_mint_path() {
            let mut app = mock_app();
            let (whoami, paths, token_id) = setup_test_case_with_name(
                &mut app,
                Some(PaymentDetails::Native {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100),
                }),
            );

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, coins(100, NATIVE_DENOM)).unwrap();

            let resp = get_nft_owner(&mut app, whoami, format!("{}::{}", token_id, path));
            assert_eq!(resp.owner, USER.to_string());
        }

        #[test]
        #[should_panic(expected = "Must send reserve token 'ujunox'")]
        fn test_mint_path_invalid_denom() {
            let mut app = mock_app();
            let (_whoami, paths, _token_id) = setup_test_case_with_name(
                &mut app,
                Some(PaymentDetails::Native {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100),
                }),
            );

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, coins(100, INVALID_DENOM)).unwrap();
        }

        #[test]
        #[should_panic(expected = "Insufficient funds sent to mint a path")]
        fn test_mint_path_pay_too_much() {
            let mut app = mock_app();
            let (_whoami, paths, _token_id) = setup_test_case_with_name(
                &mut app,
                Some(PaymentDetails::Native {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100),
                }),
            );

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, coins(101, NATIVE_DENOM)).unwrap();
        }

        #[test]
        #[should_panic(expected = "Insufficient funds sent to mint a path")]
        fn test_mint_path_pay_too_little() {
            let mut app = mock_app();
            let (_whoami, paths, _token_id) = setup_test_case_with_name(
                &mut app,
                Some(PaymentDetails::Native {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100),
                }),
            );

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, coins(99, NATIVE_DENOM)).unwrap();
        }

        #[test]
        #[should_panic(expected = "Unauthorized")]
        fn test_mint_path_pay_cw20() {
            let mut app = mock_app();
            let cw20_addr = instantiate_cw20(&mut app);
            let (_whoami, paths, _token_id) = setup_test_case_with_name(
                &mut app,
                Some(PaymentDetails::Native {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(100),
                }),
            );

            let path = "a".to_string();
            mint_path_cw20(&mut app, cw20_addr, paths, USER, Uint128::new(100), &path).unwrap();
        }

        #[test]
        #[should_panic(expected = "The root token has not been received yet")]
        fn test_mint_path_no_root_name() {
            let mut app = mock_app();
            let (_whoami, paths) = setup_test_case(&mut app, None);

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, coins(100, NATIVE_DENOM)).unwrap();
        }
    }

    mod cw20_payment {}

    mod no_payment {
        use crate::integration_tests::tests::{
            get_nft_owner, instantiate_cw20, mint_path_cw20, mint_path_native, mock_app,
            setup_test_case, setup_test_case_with_name, NATIVE_DENOM, USER,
        };
        use cosmwasm_std::{coins, Uint128};

        #[test]
        fn test_mint_path() {
            let mut app = mock_app();
            let (whoami, paths, token_id) = setup_test_case_with_name(&mut app, None);

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, vec![]).unwrap();

            let resp = get_nft_owner(&mut app, whoami, format!("{}::{}", token_id, path));
            assert_eq!(resp.owner, USER.to_string());
        }

        #[test]
        #[should_panic(expected = "This message does no accept funds")]
        fn test_mint_path_pay_native() {
            let mut app = mock_app();
            let (_whoami, paths, _token_id) = setup_test_case_with_name(&mut app, None);

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, coins(1000, NATIVE_DENOM)).unwrap();
        }

        #[test]
        #[should_panic(expected = "No payment is needed to mint a path")]
        fn test_mint_path_pay_cw20() {
            let mut app = mock_app();
            let cw20_addr = instantiate_cw20(&mut app);
            let (_whoami, paths, _token_id) = setup_test_case_with_name(&mut app, None);

            let path = "a".to_string();
            mint_path_cw20(&mut app, cw20_addr, paths, USER, Uint128::new(100), &path).unwrap();
        }

        #[test]
        #[should_panic(expected = "The root token has not been received yet")]
        fn test_mint_path_no_root_name() {
            let mut app = mock_app();
            let (_whoami, paths) = setup_test_case(&mut app, None);

            let path = "a".to_string();
            mint_path_native(&mut app, paths, USER, &path, vec![]).unwrap();
        }
    }
}
