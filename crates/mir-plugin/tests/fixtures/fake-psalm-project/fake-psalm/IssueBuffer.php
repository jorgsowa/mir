<?php
namespace Psalm;

use Psalm\Issue\CodeIssue;

final class IssueBuffer
{
    /** @var array<string, list<IssueData>> */
    private static array $issues = [];

    /** @param list<string> $suppressed */
    public static function maybeAdd(CodeIssue $e, array $suppressed = []): void
    {
        $parts = explode('\\', $e::class);
        $loc = $e->code_location;
        self::$issues[$loc->file_path][] = new IssueData(
            end($parts),
            $e->message,
            'error',
            $loc->file_start,
            $loc->file_end + 1
        );
    }

    /** @return array<string, list<IssueData>> */
    public static function clear(): array
    {
        $data = self::$issues;
        self::$issues = [];
        return $data;
    }
}
