===description===
InaccessibleClassConstant does NOT fire when a private constant is accessed within the declaring class itself.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Config {
    private const SECRET = "hidden";

    public function getSecret(): string {
        return self::SECRET;
    }
}
