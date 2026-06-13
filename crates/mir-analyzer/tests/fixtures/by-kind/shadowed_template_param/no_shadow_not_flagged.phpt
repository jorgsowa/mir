===description===
ShadowedTemplateParam does NOT fire when the method template uses a different name
than the class-level template.
===file===
<?php
/**
 * @template T
 */
class Container {
    /**
     * @template U
     * @param U $item
     * @return U
     */
    public function transform(mixed $item): mixed {
        return $item;
    }
}

===expect===
