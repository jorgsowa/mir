===description===
DeprecatedConstant fires using the accessor class name when a child class is used to access a deprecated constant inherited from a parent.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Base {
    /** @deprecated use RETRIES instead */
    const MAX_RETRIES = 5;
}

class Client extends Base {}

$v = Client::MAX_RETRIES;
===expect===
DeprecatedConstant@9:13-9:24: Constant Client::MAX_RETRIES is deprecated: use RETRIES instead
