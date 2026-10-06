===description===
variable used as dynamic property name in read is not reported
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class HasOneOrMany {
    protected function buildDictionary(array $results): array {
        $foreign = $this->getForeignKeyName();
        $dict = [];
        foreach ($results as $item) {
            $dict[$item->{$foreign}][] = $item;
        }
        return $dict;
    }

    protected function getForeignKeyName(): string { return 'user_id'; }
}
