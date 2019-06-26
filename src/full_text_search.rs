pub mod types {
    use diesel::types::{HasSqlType, NotNull};
    use diesel::pg::{Pg, PgTypeMetadata, PgMetadataLookup};

    #[derive(Clone, Copy)] pub struct TsQuery;
    #[derive(Clone, Copy)] pub struct TsVector;
    #[derive(Clone, Copy)] pub struct RegConfig;

    impl HasSqlType<TsQuery> for Pg {
        fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
            PgTypeMetadata {
                oid: 3615,
                array_oid: 3645,
            }
        }
    }

    impl HasSqlType<TsVector> for Pg {
        fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
            PgTypeMetadata {
                oid: 3614,
                array_oid: 3643,
            }
        }
    }

    impl HasSqlType<RegConfig> for Pg {
        fn metadata(_: &PgMetadataLookup) -> PgTypeMetadata {
            PgTypeMetadata {
                oid: 3734,
                array_oid: 3735,
            }
        }
    }

    impl NotNull for TsVector {}
    impl NotNull for TsQuery {}
    impl NotNull for RegConfig {}
}

mod dsl {
    use super::types::*;
    use diesel::expression::{Expression, AsExpression};

    mod predicates {
        use diesel::pg::Pg;

        diesel_infix_operator!(Matches, " @@ ", backend: Pg);
    }

    use self::predicates::*;

    pub trait TsVectorExtensions: Expression<SqlType=TsVector> + Sized {
        fn matches<T: AsExpression<TsQuery>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
    }


    pub trait TsQueryExtensions: Expression<SqlType=TsQuery> + Sized {
        fn matches<T: AsExpression<TsVector>>(self, other: T) -> Matches<Self, T::Expression> {
            Matches::new(self, other.as_expression())
        }
    }

    impl<T: Expression<SqlType=TsVector>> TsVectorExtensions for T {}

    impl<T: Expression<SqlType=TsQuery>> TsQueryExtensions for T {}
}

pub use self::dsl::*;