===description===
An elseif reached only when narrowing says earlier conditions covered every case still runs for values the docblock doesn't admit
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Finder {
    /** @param int|string $x */
    public static function find($x): string {
        if (is_int($x)) {
            return 'int';
        } elseif (is_string($x)) {
//                ^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
            return 'string';
        } elseif (is_array($x)) {
            return self::fromArray($x);
        }
        return self::fallback();
//      ^^^^^^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }

    private static function fromArray(array $x): string { return 'array'; }
//                                    ^^^^^^^^ UnusedParam: Parameter $x is never used

    private static function fallback(): string { return 'other'; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Finder::fallback() is never called
}

Finder::find(1);
