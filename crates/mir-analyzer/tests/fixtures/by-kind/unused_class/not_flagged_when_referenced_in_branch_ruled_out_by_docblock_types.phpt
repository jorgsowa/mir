===description===
A guard against values its `@param` type rules out still runs, so references there count
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class InvalidArg extends \Exception {
    public static function create(): self {
        return new self('invalid');
    }
}

final class Assert {
    /** @param array|\ArrayAccess $x */
    public static function check($x): void {
        if (!(is_array($x) || $x instanceof \ArrayAccess)) {
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
            throw InvalidArg::create();
        }
    }
}

Assert::check([]);
