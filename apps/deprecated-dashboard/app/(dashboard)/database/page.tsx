import React from "react";
import Link from "next/link";
import { Card, Table } from "@/components/ui";
import { databaseApi } from "@/lib/api";
import { formatBytes, formatNumber } from "@/lib/utils";
import type { DatabaseTable } from "@/types";
import styles from "./page.module.scss";

export default async function DatabasePage() {
  let tables: DatabaseTable[] = [];

  try {
    tables = await databaseApi.listTables();
  } catch (error) {
    console.error("Failed to fetch database tables:", error);
  }

  // Format data for display (no render functions - Server Component compatible)
  const tableData = tables.map((table) => ({
    name: table.name,
    nameLink: `/database/tables/${table.name}`,
    row_count:
      table.row_count !== undefined ? formatNumber(table.row_count) : "N/A",
    size_bytes:
      table.size_bytes !== undefined ? formatBytes(table.size_bytes) : "N/A",
  }));

  const columns = [
    {
      key: "name",
      header: "Table Name",
    },
    {
      key: "row_count",
      header: "Row Count",
    },
    {
      key: "size_bytes",
      header: "Size",
    },
  ];

  return (
    <div className={styles.database}>
      <div className={styles.header}>
        <h1>Database</h1>
        <Link href="/database/query" className={styles.queryLink}>
          Query Interface
        </Link>
      </div>

      <Card>
        <div className={styles.section}>
          <h2>Tables</h2>
          {tables.length > 0 ? (
            <div>
              <Table columns={columns} data={tableData} />
              <div className={styles.tableLinks}>
                {tables.map((table) => (
                  <Link
                    key={table.name}
                    href={`/database/tables/${table.name}`}
                    className={styles.link}
                  >
                    View {table.name} schema
                  </Link>
                ))}
              </div>
            </div>
          ) : (
            <p className={styles.empty}>No tables found</p>
          )}
        </div>
      </Card>
    </div>
  );
}
