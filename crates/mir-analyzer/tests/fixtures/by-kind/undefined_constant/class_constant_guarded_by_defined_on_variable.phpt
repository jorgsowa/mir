===description===
`defined($class.'::CONST')` proves `$class::CONST` inside the guarded branch.
===file===
<?php
final class Plain {}

final class WithIps {
    public const IPS = ['10.0.0.1'];
}

/** @return list<string> */
function ips(): array {
    $out = [];
    foreach ([Plain::class, WithIps::class] as $class) {
        if (\defined($class.'::IPS')) {
            $out = [...$out, ...$class::IPS];
        }
    }
    return $out;
}
