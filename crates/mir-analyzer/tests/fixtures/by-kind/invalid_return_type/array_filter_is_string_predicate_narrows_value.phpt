===description===
`array_filter()` with an `is_string()` predicate narrows the returned values —
same narrowing family as the `is_null()` case, generalized beyond it.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
final class Item
{
    public string|int|null $id;
    public function getId(): string|int|null
     {
        return $this->id;
     }
}

final class Collector
{
      /**
       * @param Item[] $items
       * @return string[]
       */
    public static function ids(array $items): array
     {
        return array_values(
            array_filter(
                array_map(static fn(Item $item) => $item->getId(), $items),
                static fn(string|int|null $id) => is_string($id),
               ),
            );
        }
}
