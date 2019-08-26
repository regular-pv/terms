#[macro_export]
macro_rules! term {
    ( $f:tt ( $( $sub:tt )+ ) ) => {
        {
            let mut subs = Vec::new();
            sub_terms!(subs $($sub)+);
            Term::new($f.clone(), subs)
        }
    };
    ( $f:tt () ) => {
        {
            Term::new($f.clone(), Vec::new())
        }
    };
    ( $f:tt ) => {
        {
            Term::new($f.clone(), Vec::new())
        }
    }
}

#[macro_export]
macro_rules! sub_terms {
    ( $subs:ident $f:tt ( $( $sub:tt )* ), $( $tail:tt )+ ) => {
        $subs.push(term!($f ($($sub)*)));
        sub_terms!($subs $($tail)*);
    };
    ( $subs:ident $f:tt ( $( $sub:tt )* ), ) => {
        $subs.push(term!($f ($($sub)*)));
    };
    ( $subs:ident $f:tt ( $( $sub:tt )* ) ) => {
        $subs.push(term!($f ($($sub)*)));
    };
    ( $subs:ident $f:tt, $( $tail:tt )+ ) => {
        $subs.push(term!($f));
        sub_terms!($subs $($tail)*);
    };
    ( $subs:ident $f:tt, ) => {
        $subs.push(term!($f));
    };
    ( $subs:ident $f:tt ) => {
        $subs.push(term!($f));
    };
}

#[macro_export]
macro_rules! pattern {
    ( ? $x:tt ) => {
        Pattern::var($x.clone())
    };
    ( $f:tt ( $( $sub:tt )+ ) ) => {
        {
            let mut subs = Vec::new();
            sub_patterns!(subs $($sub)+);
            Pattern::cons($f.clone(), subs)
        }
    };
    ( $f:tt () ) => {
        {
            Pattern::cons($f.clone(), Vec::new())
        }
    };
    ( $f:tt ) => {
        {
            Pattern::cons($f.clone(), Vec::new())
        }
    }
}

#[macro_export]
macro_rules! sub_patterns {
    ( $subs:ident $f:tt ( $( $sub:tt )* ), $( $tail:tt )+ ) => {
        $subs.push(pattern!($f ($($sub)*)));
        sub_patterns!($subs $($tail)*);
    };
    ( $subs:ident $f:tt ( $( $sub:tt )* ), ) => {
        $subs.push(pattern!($f ($($sub)*)));
    };
    ( $subs:ident $f:tt ( $( $sub:tt )* ) ) => {
        $subs.push(pattern!($f ($($sub)*)));
    };
    ( $subs:ident $f:tt, $( $tail:tt )+ ) => {
        $subs.push(pattern!($f));
        sub_patterns!($subs $($tail)*);
    };
    ( $subs:ident $f:tt, ) => {
        $subs.push(pattern!($f));
    };
    ( $subs:ident $f:tt ) => {
        $subs.push(pattern!($f));
    };
    ( $subs:ident ? $x:tt, $( $tail:tt )+ ) => {
        $subs.push(pattern!(? $x));
        sub_patterns!($subs $($tail)*);
    };
    ( $subs:ident ? $x:tt, ) => {
        $subs.push(pattern!(? $x));
    };
    ( $subs:ident ? $x:tt ) => {
        $subs.push(pattern!(? $x));
    };
}
