import { TestBed } from '@angular/core/testing';
import { HttpClientTestingModule, HttpTestingController } from '@angular/common/http/testing';
import { ApiService } from './api.service';

describe('ApiService Configurable Prefixes', () => {
  let service: ApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [HttpClientTestingModule],
      providers: [ApiService]
    });
    service = TestBed.inject(ApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('should initialize with default baseUrl /api/aad/v1', () => {
    expect(service.getBaseUrl()).toBe('/api/aad/v1');
  });

  it('should direct HTTP requests to the default baseUrl', () => {
    service.getAgents().subscribe();
    const req = httpMock.expectOne('/api/aad/v1/agents/search');
    expect(req.request.method).toBe('POST');
    req.flush([]);
  });

  it('should normalize trailing slashes when setBaseUrl is called', () => {
    service.setBaseUrl('/api/custom/v1/');
    expect(service.getBaseUrl()).toBe('/api/custom/v1');

    service.getAgents().subscribe();
    const req = httpMock.expectOne('/api/custom/v1/agents/search');
    expect(req.request.method).toBe('POST');
    req.flush([]);
  });

  it('should load runtime config via loadAppConfig', async () => {
    const loadPromise = service.loadAppConfig('assets/contents/app-config.json');

    const configReq = httpMock.expectOne('assets/contents/app-config.json');
    expect(configReq.request.method).toBe('GET');
    configReq.flush({ apiPath: '/api/remote-runtime/v1', logLevel: 'INFO' });

    await loadPromise;
    expect(service.getBaseUrl()).toBe('/api/remote-runtime/v1');

    service.getAgents().subscribe();
    const agentsReq = httpMock.expectOne('/api/remote-runtime/v1/agents/search');
    agentsReq.flush([]);
  });
});
