===description===
Valid PHP: `array_filter` with a `!is_null` predicate strips the null members, so
the result is a list of ints and satisfies the declared `int[]` return. mir does
not narrow the filtered-out value type, keeps `list<int|null>`, and reports it as
not matching `array<int|string, int>`.
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
