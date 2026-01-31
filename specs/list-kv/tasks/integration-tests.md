# Add Integration Tests

End-to-end tests for the list key-value pairs endpoint.

## Requirements

### Test Setup

- Use existing test infrastructure (Spanner emulator)
- Create test data with known keys and values
- Clean up test data after tests

### Test Cases

#### Basic Functionality
- [ ] `GET /kv` returns empty array when store is empty
- [ ] `GET /kv` returns all entries when store has data
- [ ] Response includes correct `total_count`
- [ ] Each entry has `key`, `value`, `created_at`, `updated_at` fields

#### Pagination
- [ ] `?limit=2` returns only 2 entries
- [ ] `?offset=1` skips first entry
- [ ] `?limit=2&offset=1` combines correctly
- [ ] `total_count` reflects total (not limited) count

#### Sorting
- [ ] Default sort is by key ascending
- [ ] `?sort=key_asc` sorts alphabetically A-Z
- [ ] `?sort=key_desc` sorts alphabetically Z-A
- [ ] `?sort=created_asc` sorts oldest first
- [ ] `?sort=created_desc` sorts newest first
- [ ] `?sort=updated_asc` sorts by update time ascending
- [ ] `?sort=updated_desc` sorts by update time descending

#### Filtering
- [ ] `?prefix=abc` returns only keys starting with "abc"
- [ ] Prefix filter works with pagination
- [ ] `total_count` reflects filtered count

#### Error Cases
- [ ] Invalid sort value returns 400
- [ ] Response includes error message for 400

## Acceptance Criteria

- [ ] All test cases pass
- [ ] Tests run against Spanner emulator
- [ ] Tests are independent (can run in any order)
