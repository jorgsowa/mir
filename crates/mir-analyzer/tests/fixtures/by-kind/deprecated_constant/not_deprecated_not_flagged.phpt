===description===
DeprecatedConstant does NOT fire for non-deprecated constants.
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
    const MAX_RETRIES = 3;
}

$v = Config::MAX_RETRIES;
===expect===
