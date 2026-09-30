===description===
DeprecatedConstant fires when accessing a deprecated constant on an interface.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
interface Flags {
    /** @deprecated use FLAG_NEW instead */
    const OLD_FLAG = 1;
}

$v = Flags::OLD_FLAG;
//          ^^^^^^^^ DeprecatedConstant: Constant Flags::OLD_FLAG is deprecated: use FLAG_NEW instead
===expect===
