===description===
property access only valid after template narrowing
===file===
<?php
class Document {
//<^^^^^^^^^^^^^^^^ MissingConstructor: Class Document has uninitialized properties but no constructor
    public string $name;
}

class Image {
//<^^^^^^^^^^^^^ MissingConstructor: Class Image has uninitialized properties but no constructor
    public int $width;
}

/**
 * @template TAsset as Document|Image
 * @param TAsset $asset
 */
function getAssetInfo(Document|Image $asset): void {
    if ($asset instanceof Document) {
        echo $asset->name;
    } elseif ($asset instanceof Image) {
//            ^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the check is redundant
        echo $asset->width;
    }
}
===expect===
