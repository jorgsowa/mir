===description===
A define('NAME', value) call inside a function/method body registers NAME as
a global constant unconditionally — mir does no call-graph reachability
analysis, so a constant defined this way must be visible everywhere, exactly
like the existing precedent for a function/class declared inside an
if(!function_exists()) guard.
===file===
<?php

// Same-function-body use: the constant is defined and read in one call.
function useTabInSameFunction(): string {
    define('MY_TAB', "\t");
    return MY_TAB;
}

// Cross-function use: a bootstrap-style function defines constants that a
// completely separate function reads — mir cannot see whether the bootstrap
// function is ever actually invoked, so the constant must still be visible.
function registerConstants(): void {
    define('APP_NAME', 'demo');
}
function readAppName(): string {
    return APP_NAME;
}

// A common real-world idiom: define() guarded by defined(), nested inside an
// `if` inside a function body — exercises the recursive control-flow scanner.
function bootstrapAppRoot(): void {
    if (!defined('APP_ROOT')) {
        define('APP_ROOT', '/var/www');
    }
}
function readAppRoot(): string {
    return APP_ROOT;
}

// define() nested several control-flow levels deep (foreach + switch + try)
// inside a function body.
/** @param list<string> $keys */
function bootstrapFromConfig(array $keys): void {
    foreach ($keys as $key) {
        switch ($key) {
            case 'timezone':
                try {
                    define('APP_TIMEZONE', 'UTC');
                } catch (\Throwable $e) {
                    // ignore
                }
                break;
        }
    }
}
function readTimezone(): string {
    return APP_TIMEZONE;
}

// A method body define() must also be visible.
class ConfigLoader {
    public function boot(): void {
        define('APP_MODE', 'production');
    }
}
function readAppMode(): string {
    return APP_MODE;
}

// A trait method body define() must also be visible.
trait BootsConfig {
    public function bootTrait(): void {
        define('TRAIT_DEFINED', 1);
    }
}
function readTraitDefined(): int {
    return TRAIT_DEFINED;
}

// An enum method body define() must also be visible.
enum Env {
    case Prod;
    public function boot(): void {
        define('ENUM_DEFINED', true);
    }
}
function readEnumDefined(): bool {
    return ENUM_DEFINED;
}
===expect===
