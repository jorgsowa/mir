===description===
A `@var string` / `@var int` on a class constant widens its declared type but not
its value, so `match` still folds the constant into the covered literals.
===file===
<?php
interface Signals {
    /** @var string */
    public const TRACE = 'trace';
    /** @var string */
    public const LOGS = 'logs';
    /** @var string */
    public const METRICS = 'metrics';
}

final class Level {
    /** @var int */
    public const LOW = 1;
    /** @var int */
    public const HIGH = 2;
    public const OFF = 0;

    /** @param 0|1|2 $level */
    public function name(int $level): string {
        return match ($level) {
            self::OFF => 'off',
            self::LOW => 'low',
            self::HIGH => 'high',
        };
    }
}

/** @param 'trace'|'logs' $signal */
function endpoint(string $signal): string {
    return match ($signal) {
        Signals::TRACE => '/v1/traces',
        Signals::LOGS => '/v1/logs',
    };
}

/** @param 'trace'|'logs'|'metrics' $signal */
function missingArm(string $signal): string {
    return match ($signal) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: "metrics"
        Signals::TRACE => '/v1/traces',
        Signals::LOGS => '/v1/logs',
    };
}
