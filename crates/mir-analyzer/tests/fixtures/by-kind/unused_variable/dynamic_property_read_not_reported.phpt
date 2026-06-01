===description===
variable used as dynamic property name in read is not reported
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
===expect===
