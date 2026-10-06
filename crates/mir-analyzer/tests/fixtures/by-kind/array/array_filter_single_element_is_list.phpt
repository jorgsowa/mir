===description===
array_filter on a one-element list yields at most [0 => x], which is a list; wider lists can leave gaps and are not.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Item {}
function maybe(): ?Item { return null; }

/** @param Closure(): list<Item> $action */
function run(Closure $action): void {}

/** @param list<Item> $items */
function takeList(array $items): void {}

function check(?Item $a, ?Item $b): void {
    run(fn(): array => array_filter([maybe()]));
    takeList(array_filter([$a]));
    takeList(array_values(array_filter([$a, $b])));
    takeList(array_filter([$a, $b]));
}
