===description===
Case-insensitive subtype matching for `implements` and the built-in enum
interfaces: `class Thing implements ifoo {}` is an `IFoo`, and an enum
satisfies a `@param unitenum` docblock parameter regardless of the case of
either spelling. Before the fix, extends_or_implements compared FQCNs
byte-exact, so both subtype relations were missed and spurious
InvalidArgument issues were emitted. The WrongCaseClass style diagnostic on
the `implements` clause is still reported.
===config===
suppress=UnusedParam
===file===
<?php
interface IFoo {}
class Thing implements ifoo {}
enum Color: string {
    case Red = 'red';
}
function takes_ifoo(IFoo $x): void {}
/** @param unitenum $x */
function accept_enum($x): void { echo get_debug_type($x); }
takes_ifoo(new Thing());
accept_enum(Color::Red);
===expect===
WrongCaseClass@3:0-3:30: Class name 'ifoo' has incorrect casing; use 'IFoo'
