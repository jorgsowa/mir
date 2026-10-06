===description===
InaccessibleClassConstant does NOT fire for public class constants.
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
    public const TIMEOUT = 30;
}

$v = Config::TIMEOUT;
