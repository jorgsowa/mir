===description===
Not all enums met
===file===
<?php
/**
 * @param "foo"|"bar" $foo
 */
function foo(string $foo): string {
    return match ($foo) {
        "foo" => "foo",
    };
}
===expect===
UnhandledMatchCondition@6:12-8:13: Unhandled match condition: "bar"
