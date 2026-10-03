===description===
@phpstan-import-type on a standalone function resolves to its definition without false positives
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

/**
 * @phpstan-type Priority = "low"|"medium"|"high"
 */
class Config {}

/**
 * @phpstan-import-type Priority from Config
 * @param Priority $level
 * @return bool
 */
function isUrgent(string $level): bool {
    return $level === "high";
}

isUrgent("high");
isUrgent("low");
===expect===
