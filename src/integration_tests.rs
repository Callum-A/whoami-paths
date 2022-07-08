#[cfg(test)]
mod tests {
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, PaymentDetails, PaymentDetailsResponse, QueryMsg, ReceiveMsg,
    };
    use crate::state::Config;
    use cosmwasm_std::{coins, to_binary, Addr, Coin, Empty, Uint128};
    use cw20::Cw20Coin;
    use cw721::OwnerOfResponse;
    use cw_multi_test::{
        next_block, App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor,
    };
    use whoami::msg::SurchargeInfo;

    const USER: &str = "addr1";
    const ADMIN: &str = "addr2";
    const NATIVE_DENOM: &str = "ujunox";

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

    fn get_config(app: &mut App, paths_addr: Addr) -> Config {
        app.wrap()
            .query_wasm_smart(paths_addr, &QueryMsg::Config {})
            .unwrap()
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
        let config = get_config(&mut app, paths.clone());
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
            whoami.clone(),
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
        transfer_name(
            &mut app,
            whoami.clone(),
            USER,
            paths.to_string(),
            token_id.clone(),
        )
        .unwrap();
    }
}
