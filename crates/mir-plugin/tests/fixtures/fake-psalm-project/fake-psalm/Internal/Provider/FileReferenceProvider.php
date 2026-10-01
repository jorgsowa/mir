<?php
namespace Psalm\Internal\Provider;

class FileReferenceProvider
{
    /** @var array<string, array<string, bool>> */
    private static array $classes = [];
    /** @var array<string, array<string, bool>> */
    private static array $members = [];

    public function addNonMethodReferenceToClass(string $source_file, string $fq_class_name_lc): void
    {
        self::$classes[$fq_class_name_lc][$source_file] = true;
    }

    public function addMethodReferenceToClassMember(string $caller, string $member_lc): void
    {
        self::$members[$member_lc][$caller] = true;
    }

    public function getAllNonMethodReferencesToClasses(): array
    {
        return self::$classes;
    }

    public function getAllMethodReferencesToClassMembers(): array
    {
        return self::$members;
    }
}
