#[macro_export]
macro_rules! hashset {
    ([$x:expr]) => {
        {
            let mut temp_set = std::collections::HashSet::new();
            for i in $x {
                temp_set.insert(i.into());
            }
            temp_set
        }
    };

    ( $( $x:expr ),* ) => {
        {
            let mut temp_set = std::collections::HashSet::new();
            $(
                temp_set.insert($x.into());
            )*
            temp_set
        }
    };
}