===description===
UndefinedProperty suppressed when property is guarded by ?? operator
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Mailable {
    protected function newQueuedJob(): mixed {
        $messageGroup = $this->messageGroup ?? null;
        return $messageGroup;
    }
}
