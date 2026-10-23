===description===
`namespace\C` resolves against the current namespace and `\C` names only the
global constant, with no fallback either way; the issue shows the resolved name.
===file===
<?php
namespace Lib {
    const LIB_C = 1;
}

namespace App {
    const APP_C = 2;

    function f(): void {
        echo namespace\APP_C;
        echo namespace\NOPE_C;
//           ^^^^^^^^^^^^^^^^ UndefinedConstant: Constant App\NOPE_C is not defined
        echo \Lib\LIB_C;
        echo \App\APP_C;
        echo APP_C;
        echo \PHP_EOL;
        echo \APP_C;
//           ^^^^^^ UndefinedConstant: Constant APP_C is not defined
    }
}

namespace App\Sub {
    function g(): void {
        echo namespace\APP_C;
//           ^^^^^^^^^^^^^^^ UndefinedConstant: Constant App\Sub\APP_C is not defined
    }
}
