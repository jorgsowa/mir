===description===
`array_filter()` with a non-null predicate narrows the returned values.
===ignore===
===config===
php_version=8.4
===file===
<?php
final class Item
{
    public ?int $id;
    public function getId(): ?int
     {
        return $this->id;
     }
}

final class Collector
{
      /**
       * @param Item[] $items
       * @return int[]
       */
    public static function ids(array $items): array
     {
        return array_values(
            array_filter(
                array_map(static fn(Item $item) => $item->getId(), $items),
                static fn(?int $id) => !is_null($id),
               ),
            );
        }
}
===expect===
