===description===
The mixed-param replacement applies to static-method assertions
(webmozart `Assert::allString()`) and `@psalm-assert-if-true` as well.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Assert {
    /** @psalm-assert array<string> $value */
    public static function allString(mixed $value): void {}

    /** @psalm-assert-if-true array<string> $value */
    public static function isStringMap(mixed $value): bool { return true; }
}

/** @param array<string, mixed> $o */
function f(array $o): void {
    Assert::allString($o);
    /** @trace $o */
    $a = 1;
//  ^^^^^^^ Trace: Type of $o is array<int|string, string>
}

/** @param array<string, mixed> $o */
function g(array $o): void {
    if (Assert::isStringMap($o)) {
        /** @trace $o */
        $a = 1;
//      ^^^^^^^ Trace: Type of $o is array<int|string, string>
    }
}
