use core::fmt;
use core::hash::Hash;

macro_rules! id_newtype {
    ($name:ident) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name(pub u32);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "({})"), self.0)
            }
        }
    };
}

id_newtype!(FileId);
id_newtype!(CrateId);
id_newtype!(DefId);

id_newtype!(HirExprId);
id_newtype!(HirItemId);

id_newtype!(TyId);
id_newtype!(EffectId);

id_newtype!(MirBodyId);
id_newtype!(MirBlockId);
id_newtype!(MirLocalId);

id_newtype!(AirFuncId);
id_newtype!(AirBlockId);
id_newtype!(AirValueId);
