===description===
FALSE POSITIVE reproducer. The inline {@inheritdoc} syntax (curly-brace form)
should also trigger docblock inheritance, the same as the tag form.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class Item {}

interface ItemLoader {
    /** @return Item */
    public function load(int $id): mixed;
}

class FileItemLoader implements ItemLoader {
    /**
     * Loads an item from disk.
     *
     * {@inheritdoc}
     */
    public function load(int $id): mixed {
        return new Item();
    }
}

function test(FileItemLoader $loader): void {
    $item = $loader->load(1);
    /** @mir-check $item is Item */
    echo get_class($item);
}
===expect===
