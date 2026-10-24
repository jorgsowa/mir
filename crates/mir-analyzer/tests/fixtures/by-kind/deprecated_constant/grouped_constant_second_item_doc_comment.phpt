===description===
Each item of a grouped constant declaration keeps its own doc comment.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Config {
    const CURRENT = 1, /** @deprecated use CURRENT */ OLD = 2;
}

$a = Config::CURRENT;
$b = Config::OLD;
//           ^^^ DeprecatedConstant: Constant Config::OLD is deprecated: use CURRENT
