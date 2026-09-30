===description===
DeprecatedConstant fires when accessing a deprecated constant declared in a trait used by a class.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
trait OldHelpers {
    /** @deprecated use NEW_LIMIT instead */
    const LIMIT = 10;
}

class Service {
    use OldHelpers;
}

$v = Service::LIMIT;
//            ^^^^^ DeprecatedConstant: Constant Service::LIMIT is deprecated: use NEW_LIMIT instead
===expect===
