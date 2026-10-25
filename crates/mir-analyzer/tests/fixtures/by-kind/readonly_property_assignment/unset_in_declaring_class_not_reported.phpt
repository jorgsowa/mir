===description===
The declaring class may unset an uninitialized readonly property (PHP's lazy-initialization
pattern); whether it is initialized is runtime state, as for assignments in its methods.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public readonly array $items;

    public function clear(): void {
        unset($this->items);
    }
}
