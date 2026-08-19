#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runtime {
    Node,
        Python,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Role {
            Support,
                Development,
                    Test,
                        Build,
                            Shipped,
                            }

                            #[derive(Debug, Clone, PartialEq, Eq)]
                            pub struct Declaration {
                                pub runtime: Runtime,
                                    pub constraint: String,
                                        pub role: Role,
                                            pub source: String,
                                            }