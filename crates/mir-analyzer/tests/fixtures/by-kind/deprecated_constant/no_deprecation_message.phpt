===description===
DeprecatedConstant fires with no message suffix when @deprecated tag has no text.
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
    /** @deprecated */
    const OLD_MAX = 3;
}

$v = Config::OLD_MAX;
//           ^^^^^^^ DeprecatedConstant: Constant Config::OLD_MAX is deprecated
===expect===
