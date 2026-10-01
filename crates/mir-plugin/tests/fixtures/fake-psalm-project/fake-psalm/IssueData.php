<?php
namespace Psalm;

final class IssueData
{
    public function __construct(
        public string $type,
        public string $message,
        public string $severity,
        public int $from,
        public int $to
    ) {
    }
}
