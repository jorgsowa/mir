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
===expect===
DeprecatedConstant@11:14-11:19: Constant Service::LIMIT is deprecated: use NEW_LIMIT instead
