===description===
DeprecatedConstant fires using the accessor class name when a child class is used to access a deprecated constant inherited from a parent.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Base {
    /** @deprecated use RETRIES instead */
    const MAX_RETRIES = 5;
}

class Client extends Base {}

$v = Client::MAX_RETRIES;
//           ^^^^^^^^^^^ DeprecatedConstant: Constant Client::MAX_RETRIES is deprecated: use RETRIES instead
