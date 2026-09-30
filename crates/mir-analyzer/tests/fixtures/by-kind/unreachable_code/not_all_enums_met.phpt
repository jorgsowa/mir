===description===
Not all enums met
===file===
<?php
/**
 * @param "foo"|"bar" $foo
 */
function foo(string $foo): string {
    return match ($foo) {
//         ^ +2:5 UnhandledMatchCondition: Unhandled match condition: "bar"
        "foo" => "foo",
    };
}
===expect===
