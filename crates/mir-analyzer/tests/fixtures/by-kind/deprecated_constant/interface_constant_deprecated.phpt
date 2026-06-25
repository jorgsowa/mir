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
===expect===
DeprecatedConstant@7:12-7:20: Constant Flags::OLD_FLAG is deprecated: use FLAG_NEW instead
