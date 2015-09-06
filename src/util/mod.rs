use std::ops::Deref;

macro_rules! trait_enum {
    (enum $name:ident : $_trait:ident { $($var:ident($ty:ty)),* }) => {
        enum $name {
            $(
                $var($ty),
            )*
        }

        impl<'a> Deref for $name {
		    type Target = ($_trait + 'a);
		    fn deref<'b>(&'b self) -> &'b $_trait {
		        match self {
                    $(& $name::$var(ref x) => x,)*
                }
		    }
		}
    }
}
