# Add Integration Tests

End-to-end tests for the list key-value pairs endpoint.

## Requirements

### Test Setup

- Use existing test infrastructure (Spanner emulator)
- Create test data with known keys and values
- Clean up test data after tests

### Test Cases

#### Basic Functionality
- [x] `GET /kv` returns empty array when store is empty
- [x] `GET /kv` returns all entries when store has data
- [x] Response includes correct `total_count`
- [x] Each entry has `key`, `value`, `created_at`, `updated_at` fields

#### Pagination
- [x] `?limit=2` returns only 2 entries
- [x] `?offset=1` skips first entry
- [x] `?limit=2&offset=1` combines correctly
- [x] `total_count` reflects total (not limited) count

#### Sorting
- [x] Default sort is by key ascending
- [x] `?sort=key_asc` sorts alphabetically A-Z
- [x] `?sort=key_desc` sorts alphabetically Z-A
- [x] `?sort=created_asc` sorts oldest first
- [x] `?sort=created_desc` sorts newest first
- [x] `?sort=updated_asc` sorts by update time ascending
- [x] `?sort=updated_desc` sorts by update time descending

#### Filtering
- [x] `?prefix=abc` returns only keys starting with "abc"
- [x] Prefix filter works with pagination
- [x] `total_count` reflects filtered count

#### Error Cases
- [x] Invalid sort value returns 400
- [x] Response includes error message for 400

## Acceptance Criteria

- [x] All test cases pass
- [x] Tests run against Spanner emulator
- [x] Tests are independent (can run in any order)
