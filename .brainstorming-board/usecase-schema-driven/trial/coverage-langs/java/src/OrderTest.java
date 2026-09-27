import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;
import static org.junit.jupiter.api.Assertions.*;
class OrderTest {
    static void scenario(String id) {
        String p = System.getenv("SCENARIO_TRACE");
        if (p != null) try { java.nio.file.Files.writeString(java.nio.file.Path.of(p), id + "\n",
            java.nio.file.StandardOpenOption.CREATE, java.nio.file.StandardOpenOption.APPEND); } catch (Exception e) {}
    }
    @ParameterizedTest @CsvSource({"0,SC-01J7Q4M", "1,SC-01J7Q4N"})
    void confirm(int n, String id) {
        scenario(id);
        Order o = new Order(); for (int i = 0; i < n; i++) o.lines.add("りんご");
        if (n == 0) assertThrows(IllegalStateException.class, o::confirm); else { o.confirm(); assertEquals("確定済", o.status); }
    }
}
