import unittest
from lab08 import checksum


class TestChecksum(unittest.TestCase):

    def test_valid(self):
        data = bytes("""
            Добавьте два-три теста, покрывающих как случаи корректной работы,
            так и случаи ошибки в данных (сбой битов). Вы можете не использовать
            тестовые фреймворки и реализовать тестовые сценарии в консольном приложении."""
                     , 'UTF-8')

        cs = checksum.calc_cs16(data)
        self.assertTrue(checksum.check_cs16(cs, data))

    def test_invalid(self):
        data = bytes("original string", 'UTF-8')
        other = bytes("other string", 'UTF-8')

        cs = checksum.calc_cs16(data)
        self.assertFalse(checksum.check_cs16(cs, other))


if __name__ == '__main__':
    unittest.main()
