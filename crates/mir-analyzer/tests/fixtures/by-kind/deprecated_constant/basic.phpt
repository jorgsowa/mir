===description===
DeprecatedConstant fires when accessing a deprecated class constant.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Config {
    /** @deprecated use MAX_RETRIES instead */
    const OLD_MAX = 3;
}

$v = Config::OLD_MAX;
//           ^^^^^^^ DeprecatedConstant: Constant Config::OLD_MAX is deprecated: use MAX_RETRIES instead
===expect===
