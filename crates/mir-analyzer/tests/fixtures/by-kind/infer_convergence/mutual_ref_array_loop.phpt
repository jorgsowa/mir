===description===
Mutually-referential classes that build arrays in loops must not cause infinite recursion.
Before the convergence fix, widen_array_with_value_and_key would emit a new TArray for
every existing TArray variant on each salsa fixpoint iteration, causing unbounded union
growth and an infinite hang.
===file:ItemList.php===
<?php
class ItemList {
    /** @return array<int, string> */
    public function labels(): array {
        $result = [];
        foreach ($this->entries() as $i => $entry) {
            $result[$i] = $entry;
        }
        return $result;
    }

    /** @return array<int, string> */
    public function entries(): array {
        return ['a', 'b', 'c'];
    }
}
===file:Processor.php===
<?php
class Processor {
    /** @return array<string, array<int, string>> */
    public function process(ItemList $list): array {
        $result = [];
        foreach ($list->labels() as $i => $label) {
            $result[$label] = $list->entries();
        }
        return $result;
    }
}
===expect===
