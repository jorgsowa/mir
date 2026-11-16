===description===
members of an @internal class are internal to the class's root namespace
===file===
<?php
namespace PhpdocAdvancedPhpstanInternalLib {
    /** @internal */
    final class Secret {
        public static function token(): string { return 'secret'; }
        public function value(): string { return 'value'; }
    }

    final class Open {
        public static function token(): string { return 'open'; }
    }

    function inside(): string {
        return Secret::token() . (new Secret())->value();
    }
}

namespace PhpdocAdvancedPhpstanInternalLib\Sub {
    function sameRoot(): string {
        return \PhpdocAdvancedPhpstanInternalLib\Secret::token();
    }
}

namespace PhpdocAdvancedPhpstanInternalApp {
    function outside(): string {
        return \PhpdocAdvancedPhpstanInternalLib\Secret::token();
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InternalMethod: Method PhpdocAdvancedPhpstanInternalLib\Secret::token() is marked @internal
    }

    function outsideInstance(\PhpdocAdvancedPhpstanInternalLib\Secret $secret): string {
        return $secret->value();
//             ^^^^^^^^^^^^^^^^ InternalMethod: Method PhpdocAdvancedPhpstanInternalLib\Secret::value() is marked @internal
    }

    function notInternal(): string {
        return \PhpdocAdvancedPhpstanInternalLib\Open::token() . \GlobalSecret::token();
    }
}

namespace {
    /** @internal */
    final class GlobalSecret {
        public static function token(): string { return 'global'; }
    }
}
