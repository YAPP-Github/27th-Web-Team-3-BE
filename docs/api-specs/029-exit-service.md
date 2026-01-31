## # 29. 서비스 탈퇴 API (회원 탈퇴)

### **DELETE** `/api/v1/members/me`

### **👉 Description**

현재 로그인한 사용자의 계정을 삭제하고 서비스를 탈퇴 처리합니다.

* 탈퇴 시 해당 사용자와 연결된 모든 개인 정보 및 데이터는 즉시 파기되며, 이는 복구가 불가능합니다.

---

### **👉 Request Header**

| Name | Type | 필수 여부 | Description |
| --- | --- | --- | --- |
| **Authorization** | String | `required` | `Bearer {accessToken}` (JWT) |

---

### **👉 Request Body**

| Field | Type | 필수 여부 | Description |
| --- | --- | --- | --- |


---

### **👉 Response Body (성공)**

| Field | Type | Description |
| --- | --- | --- |
| **isSuccess** | Boolean | `true` |
| **code** | String | `COMMON200` |
| **message** | String | "회원 탈퇴가 성공적으로 완료되었습니다." |
| **result** | Object | `null` |

**Response Body 예시:**

```json
{
  "isSuccess": true,
  "code": "COMMON200",
  "message": "회원 탈퇴가 성공적으로 완료되었습니다.",
  "result": null
}

```

---

### **👉 주요 실패 케이스**

| HTTP Status | Code | Message | 설명 |
| --- | --- | --- | --- |
| **401** | `AUTH4001` | 인증 정보가 유효하지 않습니다. | • 토큰 누락, 만료 또는 잘못된 Bearer 형식 |
| **404** | `MEMBER4042` | 존재하지 않는 사용자입니다. | • 이미 탈퇴 처리가 완료된 계정인 경우 |
| **500** | `COMMON500` | 서버 내부 오류입니다. | • 데이터 삭제 및 연관 관계 해제 중 DB 에러 발생 |

---